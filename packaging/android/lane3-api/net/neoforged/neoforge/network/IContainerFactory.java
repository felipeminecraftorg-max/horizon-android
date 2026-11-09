// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.neoforge.network;

/** NeoForge's menu factory: the menu from the data the server sent. */
@FunctionalInterface
public interface IContainerFactory<T extends net.minecraft.world.inventory.AbstractContainerMenu>
        extends net.minecraft.world.inventory.MenuType.Factory<T> {
    @Override
    T create(int windowId, net.minecraft.world.entity.player.Inventory inventory, net.minecraft.network.RegistryFriendlyByteBuf data);
}
