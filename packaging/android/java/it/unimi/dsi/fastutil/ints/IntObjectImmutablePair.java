// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package it.unimi.dsi.fastutil.ints;

/** An int and an object, fixed. */
public class IntObjectImmutablePair<V> implements IntObjectPair<V> {
    protected final int left;
    protected final V right;

    public IntObjectImmutablePair(int left, V right) {
        this.left = left;
        this.right = right;
    }

    public static <V> IntObjectImmutablePair<V> of(int left, V right) {
        return new IntObjectImmutablePair<>(left, right);
    }

    @Override
    public int leftInt() {
        return left;
    }

    @Override
    public V right() {
        return right;
    }
}
