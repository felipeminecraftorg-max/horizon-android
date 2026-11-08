// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.util;

/** A source of random numbers: Java's own generator behind it. */
public interface RandomSource {
    static RandomSource create() {
        return new Plain(new java.util.Random());
    }

    static RandomSource create(long seed) {
        return new Plain(new java.util.Random(seed));
    }

    void setSeed(long seed);

    int nextInt();

    int nextInt(int bound);

    default int nextInt(int origin, int bound) {
        if (origin >= bound) throw new IllegalArgumentException("bound - origin is non positive");
        return origin + nextInt(bound - origin);
    }

    default int nextIntBetweenInclusive(int origin, int bound) {
        return nextInt(bound - origin + 1) + origin;
    }

    long nextLong();

    boolean nextBoolean();

    float nextFloat();

    double nextDouble();

    double nextGaussian();

    default RandomSource fork() {
        return create(nextLong());
    }

    default double triangle(double mode, double deviation) {
        return mode + deviation * (nextDouble() - nextDouble());
    }

    default void consumeCount(int count) {
        for (int i = 0; i < count; i++) nextInt();
    }

    /** Java's generator as a RandomSource. */
    final class Plain implements RandomSource {
        private final java.util.Random random;

        Plain(java.util.Random random) {
            this.random = random;
        }

        public void setSeed(long seed) {
            random.setSeed(seed);
        }

        public int nextInt() {
            return random.nextInt();
        }

        public int nextInt(int bound) {
            return random.nextInt(bound);
        }

        public long nextLong() {
            return random.nextLong();
        }

        public boolean nextBoolean() {
            return random.nextBoolean();
        }

        public float nextFloat() {
            return random.nextFloat();
        }

        public double nextDouble() {
            return random.nextDouble();
        }

        public double nextGaussian() {
            return random.nextGaussian();
        }
    }
}
