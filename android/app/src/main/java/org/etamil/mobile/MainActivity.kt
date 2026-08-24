package org.etamil.mobile

import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.widget.Button
import android.widget.EditText
import android.widget.TextView
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity
import java.io.File
import java.util.concurrent.Executors

/**
 * Write a program, run it, read what it printed.
 *
 * The whole app. There is no project model, no file browser and no tabs,
 * because the thing worth proving on a phone is that the compiler is really
 * here and really runs — not that an IDE can be squeezed onto a touchscreen.
 */
class MainActivity : AppCompatActivity() {

    private lateinit var source: EditText
    private lateinit var input: EditText
    private lateinit var output: TextView
    private lateinit var run: Button

    /**
     * One thread, not a pool.
     *
     * The compiler's output capture is thread-local, and a run must begin and
     * end on one thread. A single-threaded executor also means two taps of Run
     * queue up rather than interleaving their output into one pane.
     */
    private val worker = Executors.newSingleThreadExecutor()
    private val main = Handler(Looper.getMainLooper())

    /**
     * Where `இறக்கு` looks and where a relative file path lands.
     *
     * The app's own private directory: a program written on this phone can read
     * and write the examples beside it and nothing else on the device.
     */
    private val baseDir: String by lazy { filesDir.absolutePath }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        source = findViewById(R.id.source)
        input = findViewById(R.id.input)
        output = findViewById(R.id.output)
        run = findViewById(R.id.run)

        // The compiler names itself. If this reads "unknown" the library loaded
        // but the bridge is not the one we think it is.
        title = getString(R.string.title_with_version, Etamil.version())

        unpackExamples()

        run.setOnClickListener { execute() }
        findViewById<Button>(R.id.examples).setOnClickListener { chooseExample() }
        findViewById<Button>(R.id.check).setOnClickListener { check() }
    }

    override fun onDestroy() {
        worker.shutdownNow()
        super.onDestroy()
    }

    /**
     * Errors only, nothing run.
     *
     * Cheap enough to be a button rather than a background task, but it goes to
     * the worker anyway: "cheap" is a judgement about a program somebody else
     * wrote, and a pathological one can make the lexer work for a while.
     */
    private fun check() {
        val program = source.text.toString()
        output.text = getString(R.string.checking)
        worker.execute {
            val found = Etamil.diagnostics(program)
            main.post {
                output.text = if (found.isEmpty()) {
                    getString(R.string.no_errors)
                } else {
                    found.joinToString("\n") { d ->
                        "✗ ${d.line}:${d.column} [${d.stage}] ${d.message}"
                    }
                }
            }
        }
    }

    private fun execute() {
        val program = source.text.toString()
        val answers = input.text.toString()

        // Disabled for the duration, so a second tap cannot queue a second run
        // behind the first and leave somebody watching output appear twice.
        run.isEnabled = false
        output.text = getString(R.string.running)

        worker.execute {
            val result = Etamil.run(program, answers, baseDir)
            main.post {
                run.isEnabled = true
                output.text = format(result)
            }
        }
    }

    /**
     * Output first, then the failure.
     *
     * A program that printed three lines and then failed has told you something
     * with those three lines, and putting the error above them buries it.
     */
    private fun format(result: Etamil.Result): String = buildString {
        append(result.output)
        if (!result.ok) {
            if (isNotEmpty() && !endsWith("\n")) append('\n')
            append("✗ [")
            append(result.stage ?: "run")
            append("] ")
            append(result.error ?: getString(R.string.unknown_error))
        } else if (isEmpty()) {
            // A program that runs and prints nothing looks exactly like one that
            // did not run at all, which is the more worrying of the two.
            append(getString(R.string.ran_silently))
        }
    }

    /**
     * Copy the bundled examples into the app's directory, once.
     *
     * They live in assets, which `இறக்கு` cannot see — assets are entries in the
     * APK's zip, not files with paths. Copying them out gives a program real
     * siblings it can import.
     */
    private fun unpackExamples() {
        val names = try {
            assets.list(ASSET_DIR) ?: return
        } catch (e: Exception) {
            return
        }
        for (name in names) {
            val destination = File(filesDir, name)
            // Never overwrite: an example the author has since edited is now
            // their file, and replacing it on every launch would discard work.
            if (destination.exists()) continue
            try {
                assets.open("$ASSET_DIR/$name").use { source ->
                    destination.outputStream().use { source.copyTo(it) }
                }
            } catch (e: Exception) {
                // One example that will not unpack is not worth refusing to
                // start over. The picker simply will not list it.
                continue
            }
        }
    }

    private fun chooseExample() {
        val files = filesDir.listFiles { f -> f.isFile && f.name.endsWith(".qmz") }
            ?.sortedBy { it.name }
            .orEmpty()

        if (files.isEmpty()) {
            output.text = getString(R.string.no_examples)
            return
        }

        val names = files.map { it.name }.toTypedArray()
        AlertDialog.Builder(this)
            .setTitle(R.string.pick_example)
            .setItems(names) { _, which ->
                try {
                    source.setText(files[which].readText())
                    output.text = ""
                } catch (e: Exception) {
                    output.text = getString(R.string.could_not_read, names[which])
                }
            }
            .show()
    }

    private companion object {
        const val ASSET_DIR = "examples"
    }
}
