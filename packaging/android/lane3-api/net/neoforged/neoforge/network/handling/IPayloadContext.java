// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.neoforge.network.handling;

/** Where a message came from: lane 3's come from the acting player, on the server. */
public interface IPayloadContext {
    net.minecraft.world.entity.player.Player player();

    java.util.concurrent.CompletableFuture<Void> enqueueWork(Runnable work);

    net.minecraft.network.protocol.PacketFlow flow();

    net.minecraft.network.Connection connection();

    default void reply(net.minecraft.network.protocol.common.custom.CustomPacketPayload payload) {}

    default void disconnect(net.minecraft.network.chat.Component reason) {}
}
