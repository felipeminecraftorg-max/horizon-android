// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.neoforge.common.extensions;

/** NeoForge's menu types made from a factory that reads the server's data. */
public interface IMenuTypeExtension<T> {
    static <T extends net.minecraft.world.inventory.AbstractContainerMenu> net.minecraft.world.inventory.MenuType<T> create(
            net.neoforged.neoforge.network.IContainerFactory<T> factory) {
        return new net.minecraft.world.inventory.MenuType<>(factory);
    }
}
