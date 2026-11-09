// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.neoforge.network.registration;

import net.minecraft.network.codec.StreamCodec;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.neoforged.neoforge.network.handling.IPayloadHandler;

/** Where a mod names its messages: lane 3 keeps the handlers of the ones for the server. */
public class PayloadRegistrar {
    public PayloadRegistrar versioned(String version) {
        return this;
    }

    public PayloadRegistrar optional() {
        return this;
    }

    public PayloadRegistrar executesOn(Object thread) {
        return this;
    }

    public <T extends CustomPacketPayload> PayloadRegistrar playToClient(CustomPacketPayload.Type<T> type, StreamCodec<?, T> codec, IPayloadHandler<T> handler) {
        return this;
    }

    public <T extends CustomPacketPayload> PayloadRegistrar playToServer(CustomPacketPayload.Type<T> type, StreamCodec<?, T> codec, IPayloadHandler<T> handler) {
        dev.horizon.android.lane3.Neo.handles(type, handler);
        return this;
    }

    public <T extends CustomPacketPayload> PayloadRegistrar playBidirectional(CustomPacketPayload.Type<T> type, StreamCodec<?, T> codec, IPayloadHandler<T> handler) {
        dev.horizon.android.lane3.Neo.handles(type, handler);
        return this;
    }

    public <T extends CustomPacketPayload> PayloadRegistrar commonToServer(CustomPacketPayload.Type<T> type, StreamCodec<?, T> codec, IPayloadHandler<T> handler) {
        return playToServer(type, codec, handler);
    }

    public <T extends CustomPacketPayload> PayloadRegistrar commonBidirectional(CustomPacketPayload.Type<T> type, StreamCodec<?, T> codec, IPayloadHandler<T> handler) {
        return playBidirectional(type, codec, handler);
    }
}
