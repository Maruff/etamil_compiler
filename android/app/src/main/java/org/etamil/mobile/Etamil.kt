package org.etamil.mobile

import org.json.JSONArray
import org.json.JSONObject

/**
 * The eTamil compiler, as seen from Kotlin.
 *
 * Every method here crosses into `android/rust/src/lib.rs`. The native side
 * hands back JSON rather than Java objects — see that file for why — and this
 * object is where the JSON stops: nothing above it should have to know that a
 * result was ever a string.
 *
 * The native functions are not public. A caller that got hold of one could pass
 * it the UI thread's stack and a program with an endless loop, and the only
 * symptom would be an unresponsive app.
 */
object Etamil {

    init {
        // Matches `[lib] name` in android/rust/Cargo.toml, without the `lib`
        // prefix or the `.so`. A mismatch here fails at class-load time with
        // UnsatisfiedLinkError naming the library it could not find, which is
        // at least a legible failure.
        System.loadLibrary("etamil_android")
    }

    private external fun nativeVersion(): String?
    private external fun nativeDiagnostics(source: String): String?
    private external fun nativeRun(source: String, input: String, baseDir: String): String?

    /** One error, positioned so the editor can point at it. */
    data class Diagnostic(
        val line: Int,
        val column: Int,
        val length: Int,
        /** "lex", "parse" or "type". */
        val stage: String,
        val message: String,
    )

    /** What came of one run. */
    data class Result(
        val ok: Boolean,
        val output: String,
        val error: String?,
        /** "load", "type" or "run"; null when nothing failed. */
        val stage: String?,
    )

    /**
     * The version of the compiler compiled into this APK.
     *
     * Read from the library rather than from `versionName`, so it is the version
     * that will actually produce the diagnostics, not the version somebody
     * remembered to type into the Gradle file.
     */
    fun version(): String = nativeVersion() ?: "unknown"

    /**
     * Every error the front end can find, and nothing run.
     *
     * Safe to call on every keystroke: it resolves no imports, opens no files
     * and executes nothing. It is `--check` on the command line.
     */
    fun diagnostics(source: String): List<Diagnostic> {
        val json = nativeDiagnostics(source) ?: return emptyList()
        val array = try {
            JSONArray(json)
        } catch (e: Exception) {
            // Malformed JSON from our own library is a bug, not a user error,
            // and reporting it as a diagnostic would attach it to a line of the
            // author's program that is probably fine.
            return emptyList()
        }
        return (0 until array.length()).mapNotNull { i ->
            val item = array.optJSONObject(i) ?: return@mapNotNull null
            Diagnostic(
                line = item.optInt("line", 1),
                column = item.optInt("column", 1),
                length = item.optInt("length", 1),
                stage = item.optString("stage", "lex"),
                message = item.optString("message"),
            )
        }
    }

    /**
     * Compile and run one program.
     *
     * **Not on the main thread.** A program is allowed to loop for ten million
     * instructions, open a database and make a network request, and every one of
     * those blocks the calling thread. `MainActivity` runs this on a background
     * thread; anything else that calls it must do the same.
     *
     * @param input the program's `உள்ளிடு` answers, one per line, in the order
     *   it will read them. There is nobody to type them while it runs.
     * @param baseDir where `இறக்கு` looks for modules and where a relative file
     *   path lands. Pass a directory the app owns.
     */
    fun run(source: String, input: String = "", baseDir: String): Result {
        val json = nativeRun(source, input, baseDir)
            ?: return Result(false, "", "the compiler returned nothing", "run")

        return try {
            val obj = JSONObject(json)
            Result(
                ok = obj.optBoolean("ok", false),
                output = obj.optString("output"),
                // optString turns JSON null into "null", which would be
                // displayed to somebody as though it were the error message.
                error = if (obj.isNull("error")) null else obj.optString("error"),
                stage = if (obj.isNull("stage")) null else obj.optString("stage"),
            )
        } catch (e: Exception) {
            Result(false, "", "the compiler's reply could not be read: ${e.message}", "run")
        }
    }
}
