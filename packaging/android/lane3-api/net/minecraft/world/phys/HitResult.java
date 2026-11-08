// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.phys;

public abstract class HitResult {
    public enum Type {
        MISS,
        BLOCK,
        ENTITY
    }

    public abstract Type getType();
}
