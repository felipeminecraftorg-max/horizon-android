// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.neoforge.items;

/** Item slots that can be set. */
public interface IItemHandlerModifiable extends IItemHandler {
    void setStackInSlot(int slot, net.minecraft.world.item.ItemStack stack);
}
