package molkky;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

@DisplayName("現在の得点")
class ScoreTest {
    @Nested
    @DisplayName("1回も投てきしていないとき")
    class NoThrows {
        @Test
        @DisplayName("0点になる")
        void startsAtZero() {
            assertEquals(0, new Score().value());
        }
    }

    @Nested
    @DisplayName("投てきを記録したとき")
    class RecordedThrows {
        @Test
        @DisplayName("1回目に5本倒すと5点になる")
        void scoresFiveAfterKnockingDownFiveSkittles() {
            Throw first = new Throw(new Skittle(1), new Skittle(2), new Skittle(3),
                    new Skittle(4), new Skittle(5));

            assertEquals(5, new Score(first).value());
        }

        @Test
        @DisplayName("5本倒した後に11番を1本倒すと16点になる")
        void sumsTwoThrowsToSixteen() {
            Throw first = new Throw(new Skittle(1), new Skittle(2), new Skittle(3),
                    new Skittle(4), new Skittle(5));
            Throw second = new Throw(new Skittle(11));

            assertEquals(16, new Score(first, second).value());
        }

        @Test
        @DisplayName("0点の投てきは現在の得点を変えない")
        void keepsScoreAfterMissingAllSkittles() {
            Throw first = new Throw(new Skittle(11));
            Throw missed = new Throw();

            assertEquals(11, new Score(first, missed).value());
        }

        @Test
        @DisplayName("同じ投てきを2回渡した場合も2回分合計する")
        void countsRepeatedThrowsSeparately() {
            Throw recordedThrow = new Throw(new Skittle(11));

            assertEquals(22, new Score(recordedThrow, recordedThrow).value());
        }
    }

    @Nested
    @DisplayName("不正な投てき列で得点を生成するとき")
    // 意図的なnull入力で実行時の防御を検証するため、このグループだけ静的検査を抑制する。
    @SuppressWarnings("NullAway")
    class InvalidThrows {
        @Test
        @DisplayName("null配列を拒否する")
        void rejectsNullArray() {
            assertThrows(IllegalArgumentException.class, () -> new Score((Throw[]) null));
        }

        @Test
        @DisplayName("nullの投てきだけを渡した場合は拒否する")
        void rejectsSingleNullThrow() {
            assertThrows(IllegalArgumentException.class, () -> new Score((Throw) null));
        }

        @Test
        @DisplayName("先頭がnullの投てき列を拒否する")
        void rejectsNullFirstThrow() {
            Throw recordedThrow = new Throw(new Skittle(11));

            assertThrows(IllegalArgumentException.class, () -> new Score(null, recordedThrow));
        }

        @Test
        @DisplayName("中央がnullの投てき列を拒否する")
        void rejectsNullMiddleThrow() {
            Throw first = new Throw(new Skittle(5));
            Throw last = new Throw(new Skittle(11));

            assertThrows(IllegalArgumentException.class, () -> new Score(first, null, last));
        }

        @Test
        @DisplayName("末尾がnullの投てき列を拒否する")
        void rejectsNullLastThrow() {
            Throw recordedThrow = new Throw(new Skittle(11));

            assertThrows(IllegalArgumentException.class, () -> new Score(recordedThrow, null));
        }
    }
}
