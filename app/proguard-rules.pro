# Ktor / Coroutines / Netty Proguard Rules
-dontwarn io.ktor.**
-dontwarn kotlinx.coroutines.**
-dontwarn kotlinx.atomicfu.**
-dontwarn io.netty.**
-dontwarn org.apache.log4j.**
-dontwarn org.apache.logging.log4j.**
-dontwarn org.slf4j.impl.**
-dontwarn org.conscrypt.**
-dontwarn org.eclipse.jetty.**
-dontwarn reactor.blockhound.**
-dontwarn java.lang.management.**

# CRITICAL: Keep our main classes exactly as they are
-keep class com.potatameister.vibeview.** { *; }
-keep interface com.potatameister.vibeview.** { *; }

# Keep Activity and entry points
-keep public class com.potatameister.vibeview.MainActivity { *; }
-keep public class com.potatameister.vibeview.VibeSnippet { *; }

# Keep Compose and Material 3
-keep class androidx.compose.** { *; }
-keep interface androidx.compose.** { *; }
-keep class androidx.material3.** { *; }

# General Android / Java keeps
-keepattributes Signature,Annotation,InnerClasses,EnclosingMethod
-dontnote **
