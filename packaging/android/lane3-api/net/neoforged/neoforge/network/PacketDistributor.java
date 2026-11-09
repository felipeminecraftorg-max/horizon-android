// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.neoforge.network;

import net.minecraft.network.protocol.common.custom.CustomPacketPayload;

/**
 * Sends a mod's messages: to the server, its handler runs for the acting player (lane 3's
 * sides are one game); to clients, nothing yet.
 */
public final class PacketDistributor {
    private PacketDistributor() {}

    public static void sendToServer(CustomPacketPayload payload, CustomPacketPayload... more) {
        dev.horizon.android.lane3.Neo.serverbound(payload);
        for (CustomPacketPayload p : more) dev.horizon.android.lane3.Neo.serverbound(p);
    }

    public static void sendToPlayer(net.minecraft.server.level.ServerPlayer player, CustomPacketPayload payload, CustomPacketPayload... more) {}

    public static void sendToAllPlayers(CustomPacketPayload payload, CustomPacketPayload... more) {}

    public static void sendToPlayersInDimension(net.minecraft.server.level.ServerLevel level, CustomPacketPayload payload,
            CustomPacketPayload... more) {}

    public static void sendToPlayersTrackingEntity(net.minecraft.world.entity.Entity entity, CustomPacketPayload payload,
            CustomPacketPayload... more) {}

    public static void sendToPlayersTrackingEntityAndSelf(net.minecraft.world.entity.Entity entity, CustomPacketPayload payload,
            CustomPacketPayload... more) {}
}
