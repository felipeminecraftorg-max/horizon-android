// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.neoforge.event.entity.living;

/** An event about a living entity. */
public abstract class LivingEvent extends net.neoforged.neoforge.event.entity.EntityEvent {
    protected LivingEvent(net.minecraft.world.entity.LivingEntity entity) {
        super(entity);
    }

    @Override
    public net.minecraft.world.entity.LivingEntity getEntity() {
        return (net.minecraft.world.entity.LivingEntity) super.getEntity();
    }
}
