// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.server.level;

import net.minecraft.world.entity.player.Player;

public class ServerPlayer extends Player {
    public static final ServerPlayer ACTING = new ServerPlayer();

    public ServerLevel serverLevel() {
        return ServerLevel.CURRENT;
    }
}
