# Ktor CIO / Coroutines Proguard Rules
-dontwarn io.ktor.**
-dontwarn kotlinx.coroutines.**
-dontwarn kotlinx.atomicfu.**

# CRITICAL: Keep our main classes exactly as they are
-keep class com.potatameister.vibeview.** { *; }
-keep interface com.potatameister.vibeview.** { *; }

# Keep Activity and entry points
-keep public class com.potatameister.vibeview.MainActivity
-keep public class com.potatameister.vibeview.VibeSnippet

# Keep Compose internal logic
-keep class androidx.compose.** { *; }
-keep interface androidx.compose.** { *; }

# Basic Android keeps
-keepattributes Signature,Annotation,InnerClasses,EnclosingMethod
