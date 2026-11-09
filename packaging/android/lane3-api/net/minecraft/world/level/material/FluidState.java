// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.level.material;

/** A fluid in a block's space: lane 3 answers none. */
public class FluidState {
    public boolean isEmpty() {
        return true;
    }

    public boolean isSource() {
        return false;
    }

    public int getAmount() {
        return 0;
    }
}
