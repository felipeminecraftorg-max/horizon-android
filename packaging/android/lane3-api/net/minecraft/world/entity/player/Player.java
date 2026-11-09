// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.minecraft.world.entity.player;

import net.minecraft.world.entity.LivingEntity;

public abstract class Player extends LivingEntity {
    /** The menu the player has open (its inventory's when none). */
    public net.minecraft.world.inventory.AbstractContainerMenu containerMenu;
    private final Inventory inventory = new Inventory(this);

    public Inventory getInventory() {
        return inventory;
    }

    /** Opens a mod's menu and its screen (vanilla's {@code openMenu}): its window id. */
    public java.util.OptionalInt openMenu(net.minecraft.world.MenuProvider provider) {
        return dev.horizon.android.lane3.Neo.openMenu(provider, null);
    }

    /** NeoForge's: the screen's data starts with {@code pos}. */
    public java.util.OptionalInt openMenu(net.minecraft.world.MenuProvider provider, net.minecraft.core.BlockPos pos) {
        return dev.horizon.android.lane3.Neo.openMenu(provider, buf -> buf.writeBlockPos(pos));
    }

    /** NeoForge's: the screen's data is what {@code extra} writes. */
    public java.util.OptionalInt openMenu(net.minecraft.world.MenuProvider provider,
            java.util.function.Consumer<net.minecraft.network.RegistryFriendlyByteBuf> extra) {
        return dev.horizon.android.lane3.Neo.openMenu(provider, extra);
    }

    public void closeContainer() {
        dev.horizon.android.lane3.Neo.closeScreen(true);
    }

    public boolean isCreative() {
        return dev.horizon.android.lane3.Native.creative();
    }
}
