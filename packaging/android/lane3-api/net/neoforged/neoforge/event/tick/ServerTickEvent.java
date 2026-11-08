// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.neoforge.event.tick;

import net.minecraft.server.MinecraftServer;
import net.neoforged.bus.api.Event;

/** A server tick begins ({@code Pre}) or ends ({@code Post}). */
public abstract class ServerTickEvent extends Event {
    private final MinecraftServer server;

    protected ServerTickEvent(MinecraftServer server) {
        this.server = server;
    }

    public MinecraftServer getServer() {
        return server;
    }

    public boolean hasTime() {
        return true;
    }

    public static class Pre extends ServerTickEvent {
        public Pre(MinecraftServer server) {
            super(server);
        }
    }

    public static class Post extends ServerTickEvent {
        public Post(MinecraftServer server) {
            super(server);
        }
    }
}
