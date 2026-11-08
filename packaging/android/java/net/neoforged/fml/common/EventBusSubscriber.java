// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.fml.common;

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;
import net.neoforged.api.distmarker.Dist;

/** A class whose static {@code @SubscribeEvent} methods hear the game's (or the mod's) events. */
@Retention(RetentionPolicy.RUNTIME)
@Target(ElementType.TYPE)
public @interface EventBusSubscriber {
    Dist[] value() default {Dist.CLIENT, Dist.DEDICATED_SERVER};

    String modid() default "";

    Bus bus() default Bus.GAME;

    enum Bus {
        GAME,
        MOD
    }
}
