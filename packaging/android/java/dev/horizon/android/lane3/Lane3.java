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
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import net.fabricmc.api.EnvType;
import net.fabricmc.api.ModInitializer;
import net.fabricmc.fabric.api.event.EventFactory;
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerLifecycleEvents;
import net.fabricmc.fabric.api.event.lifecycle.v1.ServerTickEvents;
import net.fabricmc.fabric.api.event.player.PlayerBlockBreakEvents;
import net.fabricmc.fabric.api.event.player.UseBlockCallback;
import net.fabricmc.loader.api.FabricLoader;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.server.level.ServerPlayer;
import net.minecraft.world.InteractionHand;
import net.minecraft.world.InteractionResult;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.phys.BlockHitResult;

/**
 * Lane 3 of the mod converter: a Java mod's own code, run by Android's runtime. Its jar
 * (renamed to Mojang's names by Rust) is turned into DEX on the phone with D8, loaded with the
 * other lane 3 mods in one class loader, and started; it then hears the events Horizon fires
 * (using and breaking blocks, server ticks) and reaches the world through {@link Native}.
 *
 * <p>Every call from a mod is caught: a mod that throws is logged and the game goes on. What a
 * mod's code needs that Horizon's API slice lacks shows as the class or member it missed.
 */
public final class Lane3 {
    private Lane3() {}

    /** The events Horizon fires; one a mod registers for that isn't here never runs. */
    private static final Set<String> FIRED = new java.util.HashSet<>(java.util.Arrays.asList(
            UseBlockCallback.class.getName(),
            PlayerBlockBreakEvents.Before.class.getName(),
            PlayerBlockBreakEvents.After.class.getName(),
            ServerTickEvents.StartTick.class.getName(),
            ServerTickEvents.EndTick.class.getName(),
            ServerTickEvents.StartWorldTick.class.getName(),
            ServerTickEvents.EndWorldTick.class.getName(),
            ServerLifecycleEvents.ServerStarting.class.getName(),
            ServerLifecycleEvents.ServerStarted.class.getName()));

    private static String current = "";
    private static final Map<String, Map<String, Integer>> REGISTERED = new LinkedHashMap<>();
    private static final Set<String> MODS = new LinkedHashSet<>();
    private static final Map<String, Integer> ERRORS = new LinkedHashMap<>();
    private static String gameDir = "";
    private static ClassLoader loader;

    public static final FabricLoader LOADER = new FabricLoader() {
        public boolean isModLoaded(String id) {
            return MODS.contains(id) || id.equals("minecraft") || id.equals("java") || id.equals("fabricloader")
                    || id.startsWith("fabric-") || id.equals("fabric-api") || id.equals("fabric");
        }

        public EnvType getEnvironmentType() {
            return EnvType.CLIENT;
        }

        public Path getConfigDir() {
            File dir = new File(gameDir, "config");
            dir.mkdirs();
            return dir.toPath();
        }

        public Path getGameDir() {
            return Paths.get(gameDir);
        }

        public boolean isDevelopmentEnvironment() {
            return false;
        }
    };

    /** A registration while mod {@code current} starts. */
    public static synchronized void registered(String what) {
        REGISTERED.computeIfAbsent(current, k -> new LinkedHashMap<>()).merge(what, 1, Integer::sum);
    }

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

    /**
     * Loads the DEX archives ({@code \n} between them) in one class loader and starts the mods:
     * lines {@code id\tclass} of their main entrypoints, libraries first. Returns a line per mod:
     * {@code id\tok\tevents} or {@code id\tmissing\tapi} or {@code id\terror\twhy}.
     */
    public static synchronized String start(String dexFiles, String mods, String dir) {
        EventFactory.clearAll();
        REGISTERED.clear();
        MODS.clear();
        ERRORS.clear();
        gameDir = dir;
        List<String> paths = new ArrayList<>();
        for (String p : dexFiles.split("\n")) {
            if (p.isEmpty()) continue;
            // Android 14 loads only read-only code files.
            new File(p).setReadOnly();
            paths.add(p);
        }
        String[] lines = mods.split("\n");
        for (String line : lines) {
            String[] parts = line.split("\t");
            if (parts.length >= 1 && !parts[0].isEmpty()) MODS.add(parts[0]);
        }
        StringBuilder report = new StringBuilder();
        try {
            loader = new DexClassLoader(String.join(File.pathSeparator, paths), null, null, Lane3.class.getClassLoader());
        } catch (Throwable t) {
            return "*\terror\t" + t;
        }
        Map<String, String> outcome = new LinkedHashMap<>();
        for (String line : lines) {
            String[] parts = line.split("\t");
            if (parts.length < 2) continue;
            String id = parts[0];
            current = id;
            try {
                Class<?> type = Class.forName(parts[1], true, loader);
                Object entry;
                try {
                    // A Kotlin object.
                    entry = type.getField("INSTANCE").get(null);
                } catch (NoSuchFieldException e) {
                    entry = type.getDeclaredConstructor().newInstance();
                }
                if (entry instanceof ModInitializer m) {
                    m.onInitialize();
                }
                outcome.putIfAbsent(id, "ok");
            } catch (Throwable t) {
                outcome.put(id, describe(t));
            } finally {
                current = "";
            }
        }
        for (Map.Entry<String, String> e : outcome.entrySet()) {
            String id = e.getKey();
            String state = e.getValue();
            report.append(id).append('\t');
            if (state.equals("ok")) {
                report.append("ok\t").append(events(id));
            } else {
                report.append(state);
            }
            report.append('\n');
        }
        MinecraftServer server = MinecraftServer.CURRENT;
        guard("server starting", () -> ServerLifecycleEvents.SERVER_STARTING.invoker().onServerStarting(server));
        guard("server started", () -> ServerLifecycleEvents.SERVER_STARTED.invoker().onServerStarted(server));
        return report.toString();
    }

    /** What a mod registered, and which of those events Horizon doesn't fire yet. */
    private static String events(String id) {
        Map<String, Integer> r = REGISTERED.get(id);
        if (r == null || r.isEmpty()) return "nothing registered";
        StringBuilder out = new StringBuilder();
        for (Map.Entry<String, Integer> e : r.entrySet()) {
            if (out.length() > 0) out.append(", ");
            String name = e.getKey();
            String simple = name.substring(name.lastIndexOf('.') + 1);
            out.append(simple).append(" x").append(e.getValue());
            if (!name.startsWith("registry ") && !FIRED.contains(name)) out.append(" (not fired yet)");
        }
        return out.toString();
    }

    /** {@code missing\t<api>} for a link error (the class or member Horizon lacks), else {@code error\t<why>}. */
    static String describe(Throwable t) {
        Throwable link = null;
        for (Throwable c = t; c != null && c != c.getCause(); c = c.getCause()) {
            if (c instanceof LinkageError || c instanceof ClassNotFoundException) link = c;
        }
        if (link != null) {
            String m = String.valueOf(link.getMessage());
            int l = m.indexOf("L");
            if (link instanceof NoClassDefFoundError || link instanceof ClassNotFoundException) {
                int semi = m.indexOf(';', l);
                String name = l >= 0 && semi > l ? m.substring(l + 1, semi).replace('/', '.') : m;
                if (m.startsWith("Failed resolution of: ")) return "missing\t" + name;
                return "missing\t" + m.replace('/', '.');
            }
            return "missing\t" + link.getClass().getSimpleName() + ": " + m;
        }
        String why = t.toString();
        return "error\t" + (why.length() > 400 ? why.substring(0, 400) : why);
    }

    private static void guard(String where, Runnable call) {
        try {
            call.run();
        } catch (Throwable t) {
            String why = describe(t);
            ERRORS.merge(where + ": " + why, 1, Integer::sum);
            if (ERRORS.get(where + ": " + why) == 1) {
                try {
                    Native.log("lane 3 " + where + ": " + why.replace('\t', ' '));
                } catch (Throwable ignored) {
                    // Outside a game.
                }
            }
        }
    }

    /** A player right-clicked a block: 0 pass, 1 used (swing), 2 used, 3 refused. */
    public static int useBlock(int x, int y, int z, int face, int hand) {
        if (loader == null) return 0;
        int[] answer = {0};
        guard("use block", () -> {
            InteractionResult r = UseBlockCallback.EVENT.invoker().interact(
                    ServerPlayer.ACTING, ServerLevel.CURRENT, InteractionHand.values()[hand & 1],
                    new BlockHitResult(new BlockPos(x, y, z), Direction.from3DDataValue(face), false));
            answer[0] = switch (r) {
                case PASS -> 0;
                case FAIL -> 3;
                default -> r.shouldSwing() ? 1 : 2;
            };
        });
        return answer[0];
    }

    /** A player breaks a block: false when a mod keeps it. */
    public static boolean beforeBreak(int x, int y, int z, String state) {
        if (loader == null) return true;
        boolean[] allowed = {true};
        guard("break block", () -> allowed[0] = PlayerBlockBreakEvents.BEFORE.invoker().beforeBlockBreak(
                ServerLevel.CURRENT, ServerPlayer.ACTING, new BlockPos(x, y, z), BlockState.of(state), null));
        if (!allowed[0]) {
            guard("break canceled", () -> PlayerBlockBreakEvents.CANCELED.invoker().onBlockBreakCanceled(
                    ServerLevel.CURRENT, ServerPlayer.ACTING, new BlockPos(x, y, z), BlockState.of(state), null));
        }
        return allowed[0];
    }

    /** A player broke a block. */
    public static void afterBreak(int x, int y, int z, String state) {
        if (loader == null) return;
        guard("block broken", () -> PlayerBlockBreakEvents.AFTER.invoker().afterBlockBreak(
                ServerLevel.CURRENT, ServerPlayer.ACTING, new BlockPos(x, y, z), BlockState.of(state), null));
    }

    /** The server's tick starts ({@code end} false) or ends. */
    public static void serverTick(boolean end) {
        if (loader == null) return;
        MinecraftServer server = MinecraftServer.CURRENT;
        if (end) {
            guard("world tick", () -> ServerTickEvents.END_WORLD_TICK.invoker().onEndTick(ServerLevel.CURRENT));
            guard("server tick", () -> ServerTickEvents.END_SERVER_TICK.invoker().onEndTick(server));
        } else {
            guard("server tick", () -> ServerTickEvents.START_SERVER_TICK.invoker().onStartTick(server));
            guard("world tick", () -> ServerTickEvents.START_WORLD_TICK.invoker().onStartTick(ServerLevel.CURRENT));
        }
    }

    /** The errors mods threw during play, one line each with how often. */
    public static synchronized String errors() {
        StringBuilder out = new StringBuilder();
        for (Map.Entry<String, Integer> e : ERRORS.entrySet()) {
            out.append(e.getKey().replace('\t', ' ')).append(" x").append(e.getValue()).append('\n');
        }
        return out.toString();
    }

    /** Leaves the world: no mod hears anything until the next start. */
    public static synchronized void stop() {
        guard("server stopping", () -> ServerLifecycleEvents.SERVER_STOPPING.invoker().onServerStopping(MinecraftServer.CURRENT));
        EventFactory.clearAll();
        loader = null;
    }
}
