// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.core;

/** A 16-block section's coordinates. */
public final class SectionPos {
    private SectionPos() {}

    public static int blockToSectionCoord(int block) {
        return block >> 4;
    }

    public static int blockToSectionCoord(double block) {
        return ((int) Math.floor(block)) >> 4;
    }

    public static int sectionToBlockCoord(int section) {
        return section << 4;
    }
}
