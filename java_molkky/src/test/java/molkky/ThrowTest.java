package molkky;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

@DisplayName("投てき")
class ThrowTest {
    @Nested
    @DisplayName("スキットルを1本倒したとき")
    class SingleSkittleThrow {
        @Test
        @DisplayName("5番のスキットルを渡すと5点になる")
        void scoresFiveForSkittleFive() {
            assertEquals(5, new Throw(new Skittle(5)).calcPoint());
        }

        @Test
        @DisplayName("1番を倒すと1点になる")
        void scoresOneForSkittleOne() {
            assertEquals(1, new Throw(new Skittle(1)).calcPoint());
        }

        @Test
        @DisplayName("12番を倒すと12点になる")
        void scoresTwelveForSkittleTwelve() {
            assertEquals(12, new Throw(new Skittle(12)).calcPoint());
        }
    }

    @Nested
    @DisplayName("複数本のスキットルを倒したとき")
    class MultipleSkittleThrow {
        @Test
        @DisplayName("5番と8番を倒すと2点になる")
        void scoresTwoForTwoSkittles() {
            assertEquals(2, new Throw(new Skittle(5), new Skittle(8)).calcPoint());
        }

        @Test
        @DisplayName("1番と5番と12番を倒すと3点になる")
        void scoresThreeForThreeSkittles() {
            assertEquals(3, new Throw(new Skittle(1), new Skittle(5), new Skittle(12)).calcPoint());
        }

        @Test
        @DisplayName("同じ番号が重複する場合は生成時に拒否する")
        void rejectsDuplicateSkittleNumbers() {
            assertThrows(IllegalArgumentException.class, () -> new Throw(new Skittle(5), new Skittle(5)));
        }
    }

    @Nested
    @DisplayName("スキットルを1本も倒さなかったとき")
    class NoSkittleThrow {
        @Test
        @DisplayName("0点になる")
        void scoresZero() {
            assertEquals(0, new Throw().calcPoint());
        }
    }

    @Nested
    @DisplayName("不正なスキットルで投てきを生成するとき")
    // 意図的なnull入力で実行時の防御を検証するため、このグループだけ静的検査を抑制する。
    @SuppressWarnings("NullAway")
    class InvalidSkittles {
        @Test
        @DisplayName("null配列は生成時に拒否する")
        void rejectsNullArray() {
            assertThrows(IllegalArgumentException.class, () -> new Throw((Skittle[]) null));
        }

        @Test
        @DisplayName("nullのスキットルは生成時に拒否する")
        void rejectsNullSkittle() {
            assertThrows(IllegalArgumentException.class, () -> new Throw((Skittle) null));
        }

        @Test
        @DisplayName("複数本の先頭がnullの場合は生成時に拒否する")
        void rejectsNullFirstSkittle() {
            assertThrows(IllegalArgumentException.class, () -> new Throw(null, new Skittle(5)));
        }

        @Test
        @DisplayName("複数本の中央がnullの場合は生成時に拒否する")
        void rejectsNullMiddleSkittle() {
            assertThrows(IllegalArgumentException.class,
                    () -> new Throw(new Skittle(1), null, new Skittle(12)));
        }

        @Test
        @DisplayName("複数本の末尾がnullの場合は生成時に拒否する")
        void rejectsNullLastSkittle() {
            assertThrows(IllegalArgumentException.class, () -> new Throw(new Skittle(5), null));
        }

        @Test
        @DisplayName("nullを複数指定した場合も生成時に拒否する")
        void rejectsRepeatedNullSkittles() {
            assertThrows(IllegalArgumentException.class, () -> new Throw(null, null));
        }
    }

    @Nested
    @DisplayName("投てきの不変性")
    class ThrowConstruction {
        @Test
        @DisplayName("入力配列を後から変更しても点数は変わらない")
        void preservesScoreWhenInputArrayChanges() {
            Skittle[] skittles = {new Skittle(5)};
            Throw thrown = new Throw(skittles);

            skittles[0] = new Skittle(12);

            assertEquals(5, thrown.calcPoint());
        }
    }
}
