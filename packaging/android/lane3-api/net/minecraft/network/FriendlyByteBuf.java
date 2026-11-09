// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.network;

/** The game's buffer over the values written to it (lane 3's sides are one game). */
public class FriendlyByteBuf extends io.netty.buffer.ByteBuf {
    /** Over {@code source}'s values: what one writes the other reads. */
    public FriendlyByteBuf(io.netty.buffer.ByteBuf source) {
        super(source == null ? new java.util.ArrayDeque<>() : source.horizonValues());
    }

    @Override
    public FriendlyByteBuf writeInt(int value) {
        super.writeInt(value);
        return this;
    }

    public FriendlyByteBuf writeVarInt(int value) {
        horizonValues.add(value);
        return this;
    }

    public int readVarInt() {
        return number(horizonValues.poll()).intValue();
    }

    public FriendlyByteBuf writeVarLong(long value) {
        horizonValues.add(value);
        return this;
    }

    public long readVarLong() {
        return number(horizonValues.poll()).longValue();
    }

    public FriendlyByteBuf writeBlockPos(net.minecraft.core.BlockPos pos) {
        horizonValues.add(pos);
        return this;
    }

    public net.minecraft.core.BlockPos readBlockPos() {
        Object v = horizonValues.poll();
        return v instanceof net.minecraft.core.BlockPos p ? p : net.minecraft.core.BlockPos.ZERO;
    }

    public FriendlyByteBuf writeUtf(String text) {
        horizonValues.add(text == null ? "" : text);
        return this;
    }

    public FriendlyByteBuf writeUtf(String text, int max) {
        return writeUtf(text);
    }

    public String readUtf() {
        Object v = horizonValues.poll();
        return v == null ? "" : v.toString();
    }

    public String readUtf(int max) {
        return readUtf();
    }

    public FriendlyByteBuf writeResourceLocation(net.minecraft.resources.ResourceLocation id) {
        horizonValues.add(id);
        return this;
    }

    public net.minecraft.resources.ResourceLocation readResourceLocation() {
        Object v = horizonValues.poll();
        return v instanceof net.minecraft.resources.ResourceLocation id ? id : net.minecraft.resources.ResourceLocation.parse(String.valueOf(v));
    }

    public FriendlyByteBuf writeUUID(java.util.UUID id) {
        horizonValues.add(id);
        return this;
    }

    public java.util.UUID readUUID() {
        Object v = horizonValues.poll();
        return v instanceof java.util.UUID id ? id : new java.util.UUID(0, 0);
    }

    public <E extends Enum<E>> FriendlyByteBuf writeEnum(E value) {
        horizonValues.add(value);
        return this;
    }

    @SuppressWarnings("unchecked")
    public <E extends Enum<E>> E readEnum(Class<E> type) {
        Object v = horizonValues.poll();
        return type.isInstance(v) ? (E) v : type.getEnumConstants()[0];
    }
}
