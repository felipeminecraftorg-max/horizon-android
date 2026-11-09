// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.inventory;

/** A kind of menu: what makes it on a client from the data the server sent. */
public class MenuType<T extends AbstractContainerMenu> {
    private final Factory<T> factory;

    public MenuType(Factory<T> factory) {
        this.factory = factory;
    }

    public MenuType(MenuSupplier<T> supplier, net.minecraft.world.flag.FeatureFlagSet flags) {
        this.factory = (id, inventory, data) -> supplier.create(id, inventory);
    }

    /** The menu a client makes, from the server's data. */
    public T create(int id, net.minecraft.world.entity.player.Inventory inventory, net.minecraft.network.RegistryFriendlyByteBuf data) {
        return factory.create(id, inventory, data);
    }

    public T create(int id, net.minecraft.world.entity.player.Inventory inventory) {
        return factory.create(id, inventory, null);
    }

    @FunctionalInterface
    public interface Factory<T extends AbstractContainerMenu> {
        T create(int id, net.minecraft.world.entity.player.Inventory inventory, net.minecraft.network.RegistryFriendlyByteBuf data);
    }

    @FunctionalInterface
    public interface MenuSupplier<T extends AbstractContainerMenu> {
        T create(int id, net.minecraft.world.entity.player.Inventory inventory);
    }
}
