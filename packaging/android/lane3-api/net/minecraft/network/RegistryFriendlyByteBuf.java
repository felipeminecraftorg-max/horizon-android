// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.network;

/** A buffer that can write registry entries. */
public class RegistryFriendlyByteBuf extends FriendlyByteBuf {
    public RegistryFriendlyByteBuf(io.netty.buffer.ByteBuf source, net.minecraft.core.RegistryAccess access) {
        super(source);
    }

    public RegistryFriendlyByteBuf(io.netty.buffer.ByteBuf source) {
        super(source);
    }

    @Override
    public RegistryFriendlyByteBuf writeInt(int value) {
        super.writeInt(value);
        return this;
    }
}
