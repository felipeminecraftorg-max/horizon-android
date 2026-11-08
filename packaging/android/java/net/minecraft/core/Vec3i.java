// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.core;

public class Vec3i implements Comparable<Vec3i> {
    public static final Vec3i ZERO = new Vec3i(0, 0, 0);
    private int x;
    private int y;
    private int z;

    public Vec3i(int x, int y, int z) {
        this.x = x;
        this.y = y;
        this.z = z;
    }

    public int getX() {
        return x;
    }

    public int getY() {
        return y;
    }

    public int getZ() {
        return z;
    }

    protected Vec3i setX(int x) {
        this.x = x;
        return this;
    }

    protected Vec3i setY(int y) {
        this.y = y;
        return this;
    }

    protected Vec3i setZ(int z) {
        this.z = z;
        return this;
    }

    public int distManhattan(Vec3i o) {
        return Math.abs(o.x - x) + Math.abs(o.y - y) + Math.abs(o.z - z);
    }

    public double distSqr(Vec3i o) {
        double dx = x - o.x, dy = y - o.y, dz = z - o.z;
        return dx * dx + dy * dy + dz * dz;
    }

    @Override
    public boolean equals(Object o) {
        return o instanceof Vec3i v && v.x == x && v.y == y && v.z == z;
    }

    @Override
    public int hashCode() {
        return (y + z * 31) * 31 + x;
    }

    @Override
    public int compareTo(Vec3i o) {
        if (y != o.y) return y - o.y;
        if (z != o.z) return z - o.z;
        return x - o.x;
    }

    @Override
    public String toString() {
        return "[" + x + ", " + y + ", " + z + "]";
    }
}
