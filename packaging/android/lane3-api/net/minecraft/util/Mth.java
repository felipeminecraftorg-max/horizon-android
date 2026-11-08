// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.util;

/** The game's small maths: what mods call of it. */
public final class Mth {
    private Mth() {}

    public static final float PI = (float) Math.PI;
    public static final float DEG_TO_RAD = PI / 180.0F;
    public static final float RAD_TO_DEG = 180.0F / PI;

    public static float sin(float a) {
        return (float) Math.sin(a);
    }

    public static float cos(float a) {
        return (float) Math.cos(a);
    }

    public static float sqrt(float a) {
        return (float) Math.sqrt(a);
    }

    public static int floor(float v) {
        return (int) Math.floor(v);
    }

    public static int floor(double v) {
        return (int) Math.floor(v);
    }

    public static int ceil(float v) {
        return (int) Math.ceil(v);
    }

    public static int ceil(double v) {
        return (int) Math.ceil(v);
    }

    public static long lfloor(double v) {
        return (long) Math.floor(v);
    }

    public static int abs(int v) {
        return Math.abs(v);
    }

    public static float abs(float v) {
        return Math.abs(v);
    }

    public static int clamp(int v, int min, int max) {
        return Math.min(Math.max(v, min), max);
    }

    public static long clamp(long v, long min, long max) {
        return Math.min(Math.max(v, min), max);
    }

    public static float clamp(float v, float min, float max) {
        return v < min ? min : Math.min(v, max);
    }

    public static double clamp(double v, double min, double max) {
        return v < min ? min : Math.min(v, max);
    }

    public static float lerp(float t, float a, float b) {
        return a + t * (b - a);
    }

    public static double lerp(double t, double a, double b) {
        return a + t * (b - a);
    }

    public static int lerpInt(float t, int a, int b) {
        return a + floor(t * (b - a));
    }

    public static float wrapDegrees(float degrees) {
        float d = degrees % 360.0F;
        if (d >= 180.0F) d -= 360.0F;
        if (d < -180.0F) d += 360.0F;
        return d;
    }

    public static double wrapDegrees(double degrees) {
        double d = degrees % 360.0;
        if (d >= 180.0) d -= 360.0;
        if (d < -180.0) d += 360.0;
        return d;
    }

    public static int wrapDegrees(int degrees) {
        int d = degrees % 360;
        if (d >= 180) d -= 360;
        if (d < -180) d += 360;
        return d;
    }

    public static int nextInt(RandomSource random, int min, int max) {
        return min >= max ? min : random.nextInt(max - min + 1) + min;
    }

    public static float nextFloat(RandomSource random, float min, float max) {
        return min >= max ? min : random.nextFloat() * (max - min) + min;
    }

    public static double nextDouble(RandomSource random, double min, double max) {
        return min >= max ? min : random.nextDouble() * (max - min) + min;
    }

    public static int square(int v) {
        return v * v;
    }

    public static float square(float v) {
        return v * v;
    }

    public static double square(double v) {
        return v * v;
    }

    public static int sign(double v) {
        return v == 0.0 ? 0 : v > 0.0 ? 1 : -1;
    }

    public static double length(double x, double y, double z) {
        return Math.sqrt(x * x + y * y + z * z);
    }

    public static float atan2(double y, double x) {
        return (float) Math.atan2(y, x);
    }
}
