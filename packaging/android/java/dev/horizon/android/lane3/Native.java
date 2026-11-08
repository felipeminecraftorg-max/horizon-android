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
}
