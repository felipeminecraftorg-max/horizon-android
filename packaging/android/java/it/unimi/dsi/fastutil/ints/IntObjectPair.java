// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package it.unimi.dsi.fastutil.ints;

/** An int and an object. */
public interface IntObjectPair<V> {
    int leftInt();

    V right();

    default Integer left() {
        return leftInt();
    }

    default int firstInt() {
        return leftInt();
    }

    default V second() {
        return right();
    }

    static <V> IntObjectPair<V> of(int left, V right) {
        return new IntObjectImmutablePair<>(left, right);
    }
}
