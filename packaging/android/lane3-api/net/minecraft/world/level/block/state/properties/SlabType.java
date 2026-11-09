// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.block.state.properties;

/** Where a slab is: the top, the bottom, or both. */
public enum SlabType {
    TOP, BOTTOM, DOUBLE;

    /** Its name in a block state. */
    public String getSerializedName() {
        return name().toLowerCase(java.util.Locale.ROOT);
    }

    @Override
    public String toString() {
        return getSerializedName();
    }
}
