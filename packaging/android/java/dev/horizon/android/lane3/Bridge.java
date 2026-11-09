// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package dev.horizon.android.lane3;

import com.android.tools.r8.CompilationMode;
import com.android.tools.r8.D8;
import com.android.tools.r8.D8Command;
import com.android.tools.r8.Diagnostic;
import com.android.tools.r8.DiagnosticsHandler;
import com.android.tools.r8.OutputMode;
import dalvik.system.DexClassLoader;
import java.io.File;
import java.lang.reflect.Method;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * The app's side of lane 3. Horizon's API slice (packaging/android/lane3-api/: the part of
 * Minecraft's, Fabric's and NeoForge's API mods call, and lane 3's runtime, {@code Lane3}) is not
 * in the app: it is loaded with the mods, in their class loader, so the converter can add to it
 * what each mod names (stand-ins, members). This class turns jars into DEX with D8 and hands the
 * game's calls to that loader's {@code Lane3}; {@link Native} stays here, where the game's
 * library is.
 */
public final class Bridge {
    private Bridge() {}

    private static Class<?> runtime;
    private static final Map<String, Method> METHODS = new HashMap<>();

    /** Jars ({@code \n} between them) to one DEX archive at {@code output}: "" or why not. */
    public static String dex(String inputs, String output) {
        StringBuilder errors = new StringBuilder();
        DiagnosticsHandler handler = new DiagnosticsHandler() {
            @Override
            public void error(Diagnostic d) {
                errors.append(d.getDiagnosticMessage()).append('\n');
            }

            @Override
            public void warning(Diagnostic d) {}

            @Override
            public void info(Diagnostic d) {}
        };
        try {
            D8Command.Builder b = D8Command.builder(handler);
            for (String jar : inputs.split("\n")) {
                if (!jar.isEmpty()) b.addProgramFiles(Paths.get(jar));
            }
            b.setMinApiLevel(Math.max(28, android.os.Build.VERSION.SDK_INT));
            b.setMode(CompilationMode.RELEASE);
            b.setOutput(Paths.get(output), OutputMode.DexIndexed);
            D8.run(b.build());
            return "";
        } catch (Throwable t) {
            String why = errors.length() > 0 ? errors.toString().trim() : String.valueOf(t);
            return why.length() > 2000 ? why.substring(0, 2000) : why;
        }
    }

    private static Object call(String name, Class<?>[] types, Object... args) throws Throwable {
        if (runtime == null) return null;
        Method m = METHODS.get(name);
        if (m == null) {
            m = runtime.getMethod(name, types);
            METHODS.put(name, m);
        }
        try {
            return m.invoke(null, args);
        } catch (java.lang.reflect.InvocationTargetException e) {
            throw e.getCause();
        }
    }

    private static final Class<?>[] NONE = {};
    private static final Class<?>[] BOOL = {boolean.class};
    private static final Class<?>[] BLOCK_USE = {int.class, int.class, int.class, int.class, int.class};
    private static final Class<?>[] BLOCK_STATE = {int.class, int.class, int.class, String.class};
    private static final Class<?>[] BLOCK_AT = {String.class, int.class, int.class, int.class};

    /**
     * Loads the DEX archives ({@code \n} between them: the mods', then the API slice's) in one
     * class loader and starts the mods there (lines {@code id\tclass}); the report is that
     * loader's {@code Lane3.start}'s.
     */
    public static synchronized String start(String dexFiles, String mods, String dir) {
        stop();
        List<String> paths = new ArrayList<>();
        for (String p : dexFiles.split("\n")) {
            if (p.isEmpty()) continue;
            // Android 14 loads only read-only code files.
            new File(p).setReadOnly();
            paths.add(p);
        }
        try {
            ClassLoader loader = new DexClassLoader(String.join(File.pathSeparator, paths), null, null, Bridge.class.getClassLoader());
            runtime = Class.forName("dev.horizon.android.lane3.Lane3", true, loader);
            METHODS.clear();
            Object report = call("start", new Class<?>[] {String.class, String.class}, mods, dir);
            return report == null ? "" : report.toString();
        } catch (Throwable t) {
            runtime = null;
            return "*\terror\tlane 3's API slice did not load: " + t;
        }
    }

    public static int useBlock(int x, int y, int z, int face, int hand) {
        try {
            Object r = call("useBlock", BLOCK_USE, x, y, z, face, hand);
            return r == null ? 0 : (Integer) r;
        } catch (Throwable t) {
            return 0;
        }
    }

    public static boolean beforeBreak(int x, int y, int z, String state) {
        try {
            Object r = call("beforeBreak", BLOCK_STATE, x, y, z, state);
            return r == null || (Boolean) r;
        } catch (Throwable t) {
            return true;
        }
    }

    public static void afterBreak(int x, int y, int z, String state) {
        try {
            call("afterBreak", BLOCK_STATE, x, y, z, state);
        } catch (Throwable t) {
            // The runtime logs a mod's errors itself.
        }
    }

    /** An entity is inside a mod's block {@code block} at x, y, z. */
    public static void entityInside(String block, int x, int y, int z) {
        try {
            call("entityInside", BLOCK_AT, block, x, y, z);
        } catch (Throwable t) {
            // As above.
        }
    }

    private static final Class<?>[] TWO = {String.class, String.class};
    private static final Class<?>[] PLACED = {String.class, int.class, int.class, int.class, String.class};

    /** The acting player went from the level {@code from} to {@code to}. */
    public static void changedDimension(String from, String to) {
        try {
            call("changedDimension", TWO, from, to);
        } catch (Throwable t) {
            // As above.
        }
    }

    /** The block state {@code state} was put at x, y, z in place of {@code old}. */
    public static void placed(String state, int x, int y, int z, String old) {
        try {
            call("placed", PLACED, state, x, y, z, old);
        } catch (Throwable t) {
            // As above.
        }
    }

    /** A tick scheduled for the block state {@code state} at x, y, z is due. */
    public static void ticked(String state, int x, int y, int z) {
        try {
            call("ticked", BLOCK_AT, state, x, y, z);
        } catch (Throwable t) {
            // As above.
        }
    }

    /** The mods' blocks with their own placement or tick: lines {@code place\tid}, {@code tick\tid}. */
    public static String hooks() {
        try {
            Object r = call("hooks", NONE);
            return r == null ? "" : r.toString();
        } catch (Throwable t) {
            return "";
        }
    }

    private static final Class<?>[] TEXT = {String.class};

    /** What the player did on a mod's screen. */
    public static void screenEvents(String events) {
        try {
            call("screenEvents", TEXT, events);
        } catch (Throwable t) {
            // As above.
        }
    }

    public static void serverTick(boolean end) {
        try {
            call("serverTick", BOOL, end);
        } catch (Throwable t) {
            // As above.
        }
    }

    public static synchronized String errors() {
        try {
            Object r = call("errors", NONE);
            return r == null ? "" : r.toString();
        } catch (Throwable t) {
            return "";
        }
    }

    public static synchronized void stop() {
        try {
            call("stop", NONE);
        } catch (Throwable t) {
            // Leaving anyway.
        }
        runtime = null;
        METHODS.clear();
    }
}
