// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package io.netty.buffer;

/**
 * Bytes to send: lane 3's sides are one game, so a buffer keeps the values written to it in
 * order, for the reads that follow.
 */
public class ByteBuf {
    /** The values, oldest first. */
    protected final java.util.ArrayDeque<Object> horizonValues;

    public ByteBuf() {
        this(new java.util.ArrayDeque<>());
    }

    protected ByteBuf(java.util.ArrayDeque<Object> values) {
        this.horizonValues = values;
    }

    /** The values written and not yet read (a buffer made over this one shares them). */
    public final java.util.ArrayDeque<Object> horizonValues() {
        return horizonValues;
    }

    public int readableBytes() {
        return horizonValues.isEmpty() ? 0 : horizonValues.size() * 4;
    }

    public boolean isReadable() {
        return !horizonValues.isEmpty();
    }

    public ByteBuf writeInt(int value) {
        horizonValues.add(value);
        return this;
    }

    public int readInt() {
        return number(horizonValues.poll()).intValue();
    }

    public ByteBuf writeByte(int value) {
        horizonValues.add((byte) value);
        return this;
    }

    public byte readByte() {
        return number(horizonValues.poll()).byteValue();
    }

    public ByteBuf writeBoolean(boolean value) {
        horizonValues.add(value);
        return this;
    }

    public boolean readBoolean() {
        Object v = horizonValues.poll();
        return v instanceof Boolean b ? b : number(v).intValue() != 0;
    }

    public ByteBuf writeLong(long value) {
        horizonValues.add(value);
        return this;
    }

    public long readLong() {
        return number(horizonValues.poll()).longValue();
    }

    public ByteBuf writeDouble(double value) {
        horizonValues.add(value);
        return this;
    }

    public double readDouble() {
        return number(horizonValues.poll()).doubleValue();
    }

    public ByteBuf writeFloat(float value) {
        horizonValues.add(value);
        return this;
    }

    public float readFloat() {
        return number(horizonValues.poll()).floatValue();
    }

    public ByteBuf writeShort(int value) {
        horizonValues.add((short) value);
        return this;
    }

    public short readShort() {
        return number(horizonValues.poll()).shortValue();
    }

    protected static Number number(Object v) {
        if (v instanceof Number n) return n;
        if (v instanceof Boolean b) return b ? 1 : 0;
        return 0;
    }
}
