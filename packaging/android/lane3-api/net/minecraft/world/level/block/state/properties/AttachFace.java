// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.block.state.properties;

/** What a block hangs on: the floor, a wall or the ceiling (buttons, levers). */
public enum AttachFace {
    FLOOR, WALL, CEILING;

    /** Its name in a block state. */
    public String getSerializedName() {
        return name().toLowerCase(java.util.Locale.ROOT);
    }

    @Override
    public String toString() {
        return getSerializedName();
    }
}
