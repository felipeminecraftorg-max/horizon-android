// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level;

/** A level the server holds. */
public interface ServerLevelAccessor extends LevelAccessor {
    net.minecraft.server.level.ServerLevel getLevel();
}
