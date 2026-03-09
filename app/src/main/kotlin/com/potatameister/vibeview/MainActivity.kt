package com.potatameister.vibeview

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Terminal
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import io.ktor.server.application.*
import io.ktor.server.engine.*
import io.ktor.server.netty.*
import io.ktor.server.request.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import java.io.File
import dalvik.system.DexClassLoader
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext

class MainActivity : ComponentActivity() {
    private var server: NettyApplicationEngine? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        val dynamicContent = mutableStateOf<(@Composable () -> Unit)?>(null)
        val statusMessage = mutableStateOf("Waiting for first push from Termux...")
        val isLive = mutableStateOf(false)
        val termuxLinked = mutableStateOf<Boolean?>(null) // null = unchecked, true = linked, false = error

        startBridgeServer(dynamicContent, statusMessage, isLive)

        setContent {
            VibeViewTheme {
                Scaffold(
                    topBar = {
                        @OptIn(ExperimentalMaterial3Api::class)
                        TopAppBar(
                            title = { Text("VibeView Shell") },
                            actions = {
                                // Termux Link Status
                                IconButton(onClick = { checkTermuxLink(termuxLinked) }) {
                                    Icon(
                                        imageVector = Icons.Default.Terminal,
                                        contentDescription = "Check Link",
                                        tint = when(termuxLinked.value) {
                                            true -> Color.Green
                                            false -> Color.Red
                                            else -> LocalContentColor.current
                                        }
                                    )
                                }
                                
                                Badge(containerColor = if (isLive.value) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.error) {
                                    Text(if (isLive.value) "LIVE" else "IDLE", color = MaterialTheme.colorScheme.onPrimary)
                                }
                                Spacer(modifier = Modifier.width(8.dp))
                            }
                        )
                    }
                ) { padding ->
                    Surface(
                        modifier = Modifier.fillMaxSize().padding(padding),
                        color = MaterialTheme.colorScheme.background
                    ) {
                        Column(
                            modifier = Modifier.fillMaxSize(),
                            verticalArrangement = Arrangement.Center,
                            horizontalAlignment = Alignment.CenterHorizontally
                        ) {
                            if (dynamicContent.value != null) {
                                dynamicContent.value!!.invoke()
                            } else {
                                Text(
                                    text = statusMessage.value,
                                    style = MaterialTheme.typography.bodyLarge,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                                
                                if (termuxLinked.value == false) {
                                    Spacer(modifier = Modifier.height(16.dp))
                                    Text(
                                        text = "Vibe CLI not detected in Termux.",
                                        color = Color.Red,
                                        style = MaterialTheme.typography.labelSmall
                                    )
                                    Button(
                                        onClick = { /* Could open a dialog with the install command */ },
                                        modifier = Modifier.padding(8.dp)
                                    ) {
                                        Text("Setup Instructions")
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    private fun checkTermuxLink(status: MutableState<Boolean?>) {
        // We use the Termux:Tasker RUN_COMMAND intent protocol
        // Note: This requires 'allow-external-apps = true' in ~/.termux/termux.properties
        try {
            val intent = Intent()
            intent.setClassName("com.termux", "com.termux.app.RunCommandService")
            intent.action = "com.termux.RUN_COMMAND"
            intent.putExtra("com.termux.RUN_COMMAND_PATH", "/data/data/com.termux/files/usr/bin/vibe")
            intent.putExtra("com.termux.RUN_COMMAND_ARGS", arrayOf("--version"))
            intent.putExtra("com.termux.RUN_COMMAND_BACKGROUND", true)
            
            startService(intent)
            
            // For MVP, we'll assume if no exception occurs, it's at least trying.
            // In a real app, we'd use a Result Receiver to get the actual exit code.
            status.value = true 
        } catch (e: Exception) {
            status.value = false
        }
    }

    private fun startBridgeServer(
        dynamicContent: MutableState<(@Composable () -> Unit)?>,
        statusMessage: MutableState<String>,
        isLive: MutableState<Boolean>
    ) {
        server = embeddedServer(Netty, port = 8888, host = "127.0.0.1") {
            routing {
                post("/push") {
                    try {
                        val dexBytes = call.receiveStream().readBytes()
                        val dexFile = File(codeCacheDir, "vibe_snippet.dex")
                        dexFile.writeBytes(dexBytes)

                        withContext(Dispatchers.Main) {
                            statusMessage.value = "Injecting new bytecode..."
                            loadAndInject(dexFile, dynamicContent)
                            isLive.value = true
                            statusMessage.value = "Live!"
                        }
                        call.respondText("OK")
                    } catch (e: Exception) {
                        withContext(Dispatchers.Main) {
                            statusMessage.value = "Error: ${e.message}"
                            isLive.value = false
                        }
                        call.respond(io.ktor.http.HttpStatusCode.InternalServerError, e.message ?: "Unknown Error")
                    }
                }
            }
        }.start(wait = false)
    }

    private fun loadAndInject(dexFile: File, dynamicContent: MutableState<(@Composable () -> Unit)?>) {
        val classLoader = DexClassLoader(
            dexFile.absolutePath,
            codeCacheDir.absolutePath,
            null,
            this.javaClass.classLoader
        )

        try {
            val clazz = classLoader.loadClass("com.potatameister.vibeview.VibeSnippet")
            val method = clazz.getDeclaredMethod("getContent")
            
            dynamicContent.value = {
                method.invoke(null)
            }
        } catch (e: Exception) {
            throw Exception("VibeSnippet not found: ${e.message}")
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        server?.stop(1000, 2000)
    }
}

@Composable
fun VibeViewTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = darkColorScheme(),
        content = content
    )
}
