// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.phys;

/** A rotation (pitch, yaw) or a 2D point. */
public class Vec2 {
    public static final Vec2 ZERO = new Vec2(0, 0);
    public final float x;
    public final float y;

    public Vec2(float x, float y) {
        this.x = x;
        this.y = y;
    }
}
