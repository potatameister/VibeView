package com.potatameister.vibeview

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
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
import kotlinx.coroutines.withContext

class MainActivity : ComponentActivity() {
    private var server: NettyApplicationEngine? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // The live state of our dynamic component
        val dynamicContent = mutableStateOf<(@Composable () -> Unit)?>(null)
        val statusMessage = mutableStateOf("Waiting for first push from Termux...")
        val isLive = mutableStateOf(false)

        // Start the local bridge server
        startBridgeServer(dynamicContent, statusMessage, isLive)

        setContent {
            VibeViewTheme {
                Scaffold(
                    topBar = {
                        @OptIn(ExperimentalMaterial3Api::class)
                        TopAppBar(
                            title = { Text("VibeView Shell") },
                            actions = {
                                Badge(containerColor = if (isLive.value) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.error) {
                                    Text(if (isLive.value) "LIVE" else "IDLE", color = MaterialTheme.colorScheme.onPrimary)
                                }
                                Spacer(modifier = Modifier.width(16.dp))
                            }
                        )
                    }
                ) { padding ->
                    Surface(
                        modifier = Modifier.fillMaxSize().padding(padding),
                        color = MaterialTheme.colorScheme.background
                    ) {
                        Box(contentAlignment = Alignment.Center) {
                            dynamicContent.value?.invoke() ?: Text(
                                text = statusMessage.value,
                                style = MaterialTheme.typography.bodyLarge,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }
                }
            }
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

        // Look for the standard VibeSnippet entry point
        try {
            val clazz = classLoader.loadClass("com.potatameister.vibeview.VibeSnippet")
            val method = clazz.getDeclaredMethod("getContent")
            
            // This assumes the snippet provides a standard @Composable function
            // Note: Real Compose hot-swapping requires a more complex "stability" wrapper
            // For MVP, we'll swap a simple UI lambda.
            dynamicContent.value = {
                // We invoke the static method from our injected class
                method.invoke(null) as? Unit
            }
        } catch (e: Exception) {
            throw Exception("Failed to find VibeSnippet.getContent(): ${e.message}")
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
