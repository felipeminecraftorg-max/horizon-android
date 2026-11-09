// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package dev.horizon.android.lane3;

/**
 * What Horizon's world answers a mod's Java code: implemented in Rust
 * (game/android_bundled/lane3.rs) over the game the event came from. Outside one (a check
 * while converting) the answers are empty.
 */
public final class Native {
    private Native() {}

    public static native String block(int x, int y, int z);

    public static native boolean setBlock(int x, int y, int z, String state, int flags);

    public static native boolean destroy(int x, int y, int z, boolean drop);

    public static native boolean sneaking();

    public static native boolean creative();

    public static native String held(int hand);

    public static native void swing(int hand);

    public static native long time();

    public static native void log(String line);

    /** Runs a command as the acting player, at the mods' permission, quietly: whether it ran. */
    public static native boolean command(String command);

    /** The acting player: "x y z yaw pitch level", or null outside a game. */
    public static native String player();

    /** Whether the acting player has the effect {@code id} ({@code minecraft:blindness}). */
    public static native boolean hasEffect(String id);

    /** A tick for the block at x, y, z in {@code delay} ticks, in the level in place. */
    public static native void scheduleTick(int x, int y, int z, int delay);

    /** The block {@code id}'s default state, every property set ({@code minecraft:oak_stairs[facing=north,...]}). */
    public static native String defaultState(String id);

    /** The block {@code id}'s properties and their values: {@code facing=north,south;lit=true,false}. */
    public static native String properties(String id);

    /** The biome at x, y, z ({@code minecraft:plains}), or "" outside a game. */
    public static native String biome(int x, int y, int z);
}
