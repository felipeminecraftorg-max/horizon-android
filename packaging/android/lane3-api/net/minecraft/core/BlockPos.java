// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.core;

public class BlockPos extends Vec3i {
    public static final BlockPos ZERO = new BlockPos(0, 0, 0);

    public BlockPos(int x, int y, int z) {
        super(x, y, z);
    }

    public BlockPos(Vec3i v) {
        this(v.getX(), v.getY(), v.getZ());
    }

    public static BlockPos containing(double x, double y, double z) {
        return new BlockPos((int) Math.floor(x), (int) Math.floor(y), (int) Math.floor(z));
    }

    public BlockPos offset(int dx, int dy, int dz) {
        return dx == 0 && dy == 0 && dz == 0 ? this : new BlockPos(getX() + dx, getY() + dy, getZ() + dz);
    }

    public BlockPos offset(Vec3i v) {
        return offset(v.getX(), v.getY(), v.getZ());
    }

    public BlockPos relative(Direction d) {
        return offset(d.getStepX(), d.getStepY(), d.getStepZ());
    }

    public BlockPos relative(Direction d, int n) {
        return offset(d.getStepX() * n, d.getStepY() * n, d.getStepZ() * n);
    }

    public BlockPos above() {
        return offset(0, 1, 0);
    }

    public BlockPos above(int n) {
        return offset(0, n, 0);
    }

    public BlockPos below() {
        return offset(0, -1, 0);
    }

    public BlockPos below(int n) {
        return offset(0, -n, 0);
    }

    public BlockPos north() {
        return offset(0, 0, -1);
    }

    public BlockPos south() {
        return offset(0, 0, 1);
    }

    public BlockPos west() {
        return offset(-1, 0, 0);
    }

    public BlockPos east() {
        return offset(1, 0, 0);
    }

    public BlockPos immutable() {
        return this;
    }

    public long asLong() {
        return ((long) getX() & 0x3FFFFFFL) << 38 | ((long) getY() & 0xFFFL) | ((long) getZ() & 0x3FFFFFFL) << 12;
    }

    public static BlockPos of(long packed) {
        return new BlockPos((int) (packed >> 38), (int) (packed << 52 >> 52), (int) (packed << 26 >> 38));
    }
}
