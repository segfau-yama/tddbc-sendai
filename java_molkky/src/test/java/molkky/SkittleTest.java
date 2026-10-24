package molkky;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

@DisplayName("スキットル")
class SkittleTest {
    @Nested
    @DisplayName("有効な番号で生成するとき")
    class ValidNumbers {
        @Test
        @DisplayName("番号1を保持する")
        void storesNumberOne() {
            assertEquals(1, new Skittle(1).number());
        }

        @Test
        @DisplayName("番号12を保持する")
        void storesNumberTwelve() {
            assertEquals(12, new Skittle(12).number());
        }
    }

    @Nested
    @DisplayName("範囲外の番号で生成するとき")
    class OutOfRangeNumbers {
        @Test
        @DisplayName("番号0を拒否する")
        void rejectsNumberZero() {
            assertThrows(IllegalArgumentException.class, () -> new Skittle(0));
        }

        @Test
        @DisplayName("番号13を拒否する")
        void rejectsNumberThirteen() {
            assertThrows(IllegalArgumentException.class, () -> new Skittle(13));
        }
    }
}
