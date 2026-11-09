// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.neoforge.network.handling;

/** What a mod does with one of its messages. */
@FunctionalInterface
public interface IPayloadHandler<T extends net.minecraft.network.protocol.common.custom.CustomPacketPayload> {
    void handle(T payload, IPayloadContext context);
}
