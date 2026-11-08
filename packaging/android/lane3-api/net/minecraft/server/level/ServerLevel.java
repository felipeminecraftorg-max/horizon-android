// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.server.level;

import net.minecraft.world.level.Level;

public class ServerLevel extends Level {
    public static final ServerLevel CURRENT = new ServerLevel();

    public net.minecraft.server.MinecraftServer getServer() {
        return net.minecraft.server.MinecraftServer.CURRENT;
    }
}
