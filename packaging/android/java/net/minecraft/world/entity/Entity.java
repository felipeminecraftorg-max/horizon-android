// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.entity;

import dev.horizon.android.lane3.Native;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.Level;

/** The player an event acts for, through Horizon. */
public abstract class Entity {
    public Level level() {
        return ServerLevel.CURRENT;
    }

    public boolean isShiftKeyDown() {
        return Native.sneaking();
    }

    public boolean isCrouching() {
        return Native.sneaking();
    }

    public boolean isSpectator() {
        return false;
    }
}
