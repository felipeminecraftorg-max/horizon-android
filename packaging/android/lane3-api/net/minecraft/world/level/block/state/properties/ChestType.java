// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.block.state.properties;

/** A chest alone or a side of a double one. */
public enum ChestType {
    SINGLE, LEFT, RIGHT;

    /** Its name in a block state. */
    public String getSerializedName() {
        return name().toLowerCase(java.util.Locale.ROOT);
    }

    @Override
    public String toString() {
        return getSerializedName();
    }
}
