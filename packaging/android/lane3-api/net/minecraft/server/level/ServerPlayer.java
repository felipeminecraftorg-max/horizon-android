// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.server.level;

import net.minecraft.world.entity.player.Player;

public class ServerPlayer extends Player {
    /** The player's connection: a teleport through it moves the player. */
    public final net.minecraft.server.network.ServerGamePacketListenerImpl connection = new net.minecraft.server.network.ServerGamePacketListenerImpl();

    public static final ServerPlayer ACTING = new ServerPlayer();

    public ServerLevel serverLevel() {
        return ServerLevel.CURRENT;
    }
}
