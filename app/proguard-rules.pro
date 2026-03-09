# Netty / Ktor Proguard Rules
# Ignore missing optional dependencies that we don't use
-dontwarn io.netty.**
-dontwarn io.ktor.**
-dontwarn org.apache.log4j.**
-dontwarn org.apache.logging.log4j.**
-dontwarn org.slf4j.impl.**
-dontwarn org.conscrypt.**
-dontwarn org.eclipse.jetty.**
-dontwarn reactor.blockhound.**
-dontwarn java.lang.management.**

# Keep our Dynamic Injection entry point
-keep class com.potatameister.vibeview.VibeSnippet { *; }

# Basic Compose Keep Rules
-keep class androidx.compose.** { *; }
