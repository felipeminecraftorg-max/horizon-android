// Horizon Android, lane 3 of the mod converter: written for Horizon (the repository's licence).
package net.neoforged.neoforge.attachment;

import java.util.function.Function;
import java.util.function.Supplier;

/** Data a mod keeps on an entity, a block entity or a level, made fresh where there is none. */
public final class AttachmentType<T> {
    private final Function<Object, T> make;

    private AttachmentType(Function<Object, T> make) {
        this.make = make;
    }

    /** A fresh value for {@code holder}. */
    public T horizonMake(Object holder) {
        return make.apply(holder);
    }

    public static <T> Builder<T> builder(Supplier<T> make) {
        return new Builder<>(h -> make.get());
    }

    public static <T> Builder<T> builder(Function<Object, T> make) {
        return new Builder<>(make);
    }

    public static <T> Builder<T> serializable(Supplier<T> make) {
        return builder(make);
    }

    public static <T> Builder<T> serializable(Function<Object, T> make) {
        return builder(make);
    }

    public static final class Builder<T> {
        private final Function<Object, T> make;

        private Builder(Function<Object, T> make) {
            this.make = make;
        }

        public Builder<T> serialize(Object serializer) {
            return this;
        }

        public Builder<T> serialize(Object codec, Object predicate) {
            return this;
        }

        public Builder<T> copyOnDeath() {
            return this;
        }

        public Builder<T> copyHandler(Object handler) {
            return this;
        }

        public Builder<T> sync(Object codec) {
            return this;
        }

        public Builder<T> sync(Object predicate, Object codec) {
            return this;
        }

        public AttachmentType<T> build() {
            return new AttachmentType<>(make);
        }
    }
}
