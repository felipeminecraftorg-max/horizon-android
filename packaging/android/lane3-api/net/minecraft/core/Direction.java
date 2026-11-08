// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.core;

public enum Direction {
    DOWN(0, -1, 0),
    UP(0, 1, 0),
    NORTH(0, 0, -1),
    SOUTH(0, 0, 1),
    WEST(-1, 0, 0),
    EAST(1, 0, 0);

    private final int x;
    private final int y;
    private final int z;

    Direction(int x, int y, int z) {
        this.x = x;
        this.y = y;
        this.z = z;
    }

    public int getStepX() {
        return x;
    }

    public int getStepY() {
        return y;
    }

    public int getStepZ() {
        return z;
    }

    public int get3DDataValue() {
        return ordinal();
    }

    public static Direction from3DDataValue(int i) {
        return values()[Math.floorMod(i, 6)];
    }

    public Direction getOpposite() {
        return from3DDataValue(ordinal() ^ 1);
    }

    public String getName() {
        return name().toLowerCase(java.util.Locale.ROOT);
    }
}
