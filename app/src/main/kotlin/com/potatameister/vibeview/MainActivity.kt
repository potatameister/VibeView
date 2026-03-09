package com.potatameister.vibeview

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Terminal
import androidx.compose.material.icons.outlined.Terminal
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import io.ktor.server.application.*
import io.ktor.server.engine.*
import io.ktor.server.cio.*
import io.ktor.server.request.*
import io.ktor.server.response.*
import io.ktor.server.routing.*
import java.io.File
import dalvik.system.DexClassLoader
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class MainActivity : ComponentActivity() {
    private var server: CIOApplicationEngine? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        val dynamicContent = mutableStateOf<(@Composable () -> Unit)?>(null)
        val statusMessage = mutableStateOf("Ready for connection")
        val isLive = mutableStateOf(false)
        val termuxLinked = mutableStateOf<Boolean?>(null)

        startBridgeServer(dynamicContent, statusMessage, isLive)

        setContent {
            VibeViewTheme {
                MainScreen(
                    dynamicContent = dynamicContent.value,
                    statusMessage = statusMessage.value,
                    isLive = isLive.value,
                    termuxLinked = termuxLinked.value,
                    onCheckLink = { checkTermuxLink(termuxLinked) }
                )
            }
        }
    }

    private fun checkTermuxLink(status: MutableState<Boolean?>) {
        try {
            val intent = Intent().apply {
                setClassName("com.termux", "com.termux.app.RunCommandService")
                action = "com.termux.RUN_COMMAND"
                putExtra("com.termux.RUN_COMMAND_PATH", "/data/data/com.termux/files/usr/bin/vibe")
                putExtra("com.termux.RUN_COMMAND_ARGS", arrayOf("--version"))
                putExtra("com.termux.RUN_COMMAND_BACKGROUND", true)
            }
            startService(intent)
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
        server = embeddedServer(CIO, port = 8888, host = "127.0.0.1") {
            routing {
                post("/push") {
                    try {
                        val dexBytes = call.receiveStream().readBytes()
                        val dexFile = File(codeCacheDir, "vibe_snippet.dex")
                        dexFile.writeBytes(dexBytes)

                        withContext(Dispatchers.Main) {
                            statusMessage.value = "Injecting..."
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
                        call.respond(io.ktor.http.HttpStatusCode.InternalServerError, e.message ?: "Error")
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
            dynamicContent.value = { method.invoke(null) }
        } catch (e: Exception) {
            throw Exception("VibeSnippet not found")
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        server?.stop(500, 1000)
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainScreen(
    dynamicContent: (@Composable () -> Unit)?,
    statusMessage: String,
    isLive: Boolean,
    termuxLinked: Boolean?,
    onCheckLink: () -> Unit
) {
    Scaffold(
        topBar = {
            CenterAlignedTopAppBar(
                title = { 
                    Text(
                        "VIBEVIEW", 
                        style = MaterialTheme.typography.titleSmall,
                        letterSpacing = 2.sp,
                        fontWeight = FontWeight.Black
                    ) 
                },
                actions = {
                    IconButton(onClick = onCheckLink) {
                        Icon(
                            imageVector = if (termuxLinked == true) Icons.Default.Terminal else Icons.Outlined.Terminal,
                            contentDescription = "Link Status",
                            tint = when(termuxLinked) {
                                true -> MaterialTheme.colorScheme.primary
                                false -> MaterialTheme.colorScheme.error
                                else -> MaterialTheme.colorScheme.onSurfaceVariant
                            }
                        )
                    }
                },
                colors = TopAppBarDefaults.centerAlignedTopAppBarColors(
                    containerColor = Color.Transparent
                )
            )
        }
    ) { padding ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .background(MaterialTheme.colorScheme.background),
            contentAlignment = Alignment.Center
        ) {
            if (dynamicContent != null) {
                dynamicContent()
            } else {
                Column(
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(16.dp)
                ) {
                    StatusIndicator(isLive)
                    
                    Text(
                        text = statusMessage,
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )

                    if (termuxLinked == false) {
                        SetupInstructions()
                    }
                }
            }
        }
    }
}

@Composable
fun StatusIndicator(isLive: Boolean) {
    val infiniteTransition = rememberInfiniteTransition(label = "pulse")
    val alpha by infiniteTransition.animateFloat(
        initialValue = 0.4f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(
            animation = tween(1000, easing = FastOutSlowInEasing),
            repeatMode = RepeatMode.Reverse
        ),
        label = "alpha"
    )

    Box(
        modifier = Modifier
            .size(12.dp)
            .clip(CircleShape)
            .background(
                if (isLive) MaterialTheme.colorScheme.primary 
                else MaterialTheme.colorScheme.error.copy(alpha = alpha)
            )
    )
}

@Composable
fun SetupInstructions() {
    val clipboardManager = LocalClipboardManager.current
    val installCommand = "curl -sL https://raw.githubusercontent.com/potatameister/VibeView/main/vibe-install.sh | bash"
    
    Card(
        modifier = Modifier.padding(24.dp),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f)
        )
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Text(
                "CLI NOT DETECTED",
                style = MaterialTheme.typography.labelLarge,
                color = MaterialTheme.colorScheme.error
            )
            Spacer(modifier = Modifier.height(8.dp))
            Text(
                "Paste this into Termux to setup:",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Spacer(modifier = Modifier.height(12.dp))
            Surface(
                color = MaterialTheme.colorScheme.surface,
                shape = MaterialTheme.shapes.small
            ) {
                Text(
                    installCommand,
                    modifier = Modifier.padding(8.dp),
                    style = MaterialTheme.typography.labelSmall,
                    fontFamily = FontFamily.Monospace,
                    maxLines = 1
                )
            }
            Button(
                onClick = { clipboardManager.setText(AnnotatedString(installCommand)) },
                modifier = Modifier.padding(top = 16.dp),
                shape = MaterialTheme.shapes.medium
            ) {
                Text("Copy Setup Command")
            }
        }
    }
}

@Composable
fun VibeViewTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = darkColorScheme(
            primary = Color(0xFFD0BCFF),
            secondary = Color(0xFFCCC2DC),
            tertiary = Color(0xFFEFB8C8),
            background = Color(0xFF1C1B1F),
            surface = Color(0xFF1C1B1F),
            onPrimary = Color(0xFF381E72),
            onSecondary = Color(0xFF332D41),
            onTertiary = Color(0xFF492532),
            onBackground = Color(0xFFE6E1E5),
            onSurface = Color(0xFFE6E1E5),
            surfaceVariant = Color(0xFF49454F),
            onSurfaceVariant = Color(0xFFCAC4D0),
            error = Color(0xFFF2B8B5)
        ),
        typography = Typography(),
        content = content
    )
}
