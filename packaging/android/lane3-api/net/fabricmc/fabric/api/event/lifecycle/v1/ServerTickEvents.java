// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.fabricmc.fabric.api.event.lifecycle.v1;

import net.fabricmc.fabric.api.event.Event;
import net.fabricmc.fabric.api.event.EventFactory;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;

public final class ServerTickEvents {
    private ServerTickEvents() {}

    public static final Event<StartTick> START_SERVER_TICK = EventFactory.createArrayBacked(StartTick.class,
            listeners -> server -> {
                for (StartTick l : listeners) {
                    l.onStartTick(server);
                }
            });
    public static final Event<EndTick> END_SERVER_TICK = EventFactory.createArrayBacked(EndTick.class,
            listeners -> server -> {
                for (EndTick l : listeners) {
                    l.onEndTick(server);
                }
            });
    public static final Event<StartWorldTick> START_WORLD_TICK = EventFactory.createArrayBacked(StartWorldTick.class,
            listeners -> world -> {
                for (StartWorldTick l : listeners) {
                    l.onStartTick(world);
                }
            });
    public static final Event<EndWorldTick> END_WORLD_TICK = EventFactory.createArrayBacked(EndWorldTick.class,
            listeners -> world -> {
                for (EndWorldTick l : listeners) {
                    l.onEndTick(world);
                }
            });

    @FunctionalInterface
    public interface StartTick {
        void onStartTick(MinecraftServer server);
    }

    @FunctionalInterface
    public interface EndTick {
        void onEndTick(MinecraftServer server);
    }

    @FunctionalInterface
    public interface StartWorldTick {
        void onStartTick(ServerLevel world);
    }

    @FunctionalInterface
    public interface EndWorldTick {
        void onEndTick(ServerLevel world);
    }
}
