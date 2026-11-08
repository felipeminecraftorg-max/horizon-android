// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.network.codec;

/** How a message is written to the network and read back. */
public interface StreamCodec<B, V> {
    V decode(B buffer);

    void encode(B buffer, V value);
}
