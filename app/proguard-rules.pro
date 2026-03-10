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

# --- THE FIX FOR THE COMPILER ---
# We must NOT obfuscate the libraries we use as a classpath
# Otherwise the compiler can't find 'RoundedCornerShape' because it was renamed to 'a'
-keep class androidx.compose.** { *; }
-keep interface androidx.compose.** { *; }
-keep class androidx.material3.** { *; }
-keep interface androidx.material3.** { *; }
-keep class androidx.foundation.** { *; }
-keep interface androidx.foundation.** { *; }
-keep class androidx.runtime.** { *; }
-keep interface androidx.runtime.** { *; }
-keep class androidx.ui.** { *; }
-keep interface androidx.ui.** { *; }

# General Android / Java keeps
-keepattributes Signature,Annotation,InnerClasses,EnclosingMethod,SourceFile,LineNumberTable
-dontnote **
