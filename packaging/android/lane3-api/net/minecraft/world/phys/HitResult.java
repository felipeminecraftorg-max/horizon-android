// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.phys;

public abstract class HitResult {
    protected final Vec3 location;

    protected HitResult(Vec3 location) {
        this.location = location == null ? Vec3.ZERO : location;
    }

    /** Where the hit is. */
    public Vec3 getLocation() {
        return location;
    }

    public enum Type {
        MISS,
        BLOCK,
        ENTITY
    }

    public abstract Type getType();
}
