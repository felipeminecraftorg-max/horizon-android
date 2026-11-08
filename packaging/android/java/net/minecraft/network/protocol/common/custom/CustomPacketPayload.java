// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.network.protocol.common.custom;

import net.minecraft.resources.ResourceLocation;

/** A mod's own network message: the game's server and window share a process, so lane 3 sends none. */
public interface CustomPacketPayload {
    Type<? extends CustomPacketPayload> type();

    static <T extends CustomPacketPayload> Type<T> createType(String id) {
        return new Type<>(ResourceLocation.parse(id));
    }

    final class Type<T extends CustomPacketPayload> {
        private final ResourceLocation id;

        public Type(ResourceLocation id) {
            this.id = id;
        }

        public ResourceLocation id() {
            return id;
        }
    }
}
