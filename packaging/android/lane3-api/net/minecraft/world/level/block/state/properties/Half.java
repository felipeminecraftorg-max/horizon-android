// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.block.state.properties;

/** A half of a block: the top or the bottom (stairs, trapdoors). */
public enum Half {
    TOP, BOTTOM;

    /** Its name in a block state. */
    public String getSerializedName() {
        return name().toLowerCase(java.util.Locale.ROOT);
    }

    @Override
    public String toString() {
        return getSerializedName();
    }
}
