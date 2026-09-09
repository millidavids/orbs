//! Samples, as tensors.
//!
//! Kept apart from both the model and the trainer because inference needs it
//! too: a line typed at the prompt becomes a batch of one by exactly this
//! route, so there is no second way for a sentence to reach the reader and
//! nothing for the two to disagree about.

use burn::prelude::*;

use crate::Sample;

/// A batch of encoded sentences, ready to read.
#[derive(Debug, Clone)]
pub struct Batch<B: Backend> {
    /// `[sentences, MAX_LEN]` of vocabulary rows.
    pub tokens: Tensor<B, 2, Int>,
    /// `[sentences, MAX_LEN]`, true where the row is padding.
    pub pad: Tensor<B, 2, Bool>,
    /// `[sentences]` — which verb each one is.
    ///
    /// Meaningless for a sentence that asks for nothing; see
    /// [`commands`](Self::commands).
    pub verbs: Tensor<B, 1, Int>,
    /// `[sentences]` — 1 where the sentence is asking for something.
    ///
    /// What the binary head is trained against. Separate from `verbs` because
    /// *whether* and *which* stopped sharing a distribution (§19).
    pub commands: Tensor<B, 1, Int>,
    /// `[sentences, MAX_LEN]` — what each word is doing.
    pub tags: Tensor<B, 2, Int>,
    /// `[sentences, MAX_LEN]`, true where a row is a real word.
    ///
    /// **The metric needs this even when the loss does not.** A sentence is four
    /// words in a thirty-two row buffer, so tagging accuracy counted over the
    /// whole buffer is seven-eighths a measure of how well the reader predicts
    /// padding — which it learns in the first minute and which nobody cares
    /// about.
    pub real: Tensor<B, 2, Bool>,
}

impl<B: Backend> Batch<B> {
    /// Gather samples into one batch on `device`.
    ///
    /// # Panics
    ///
    /// If `samples` is empty — a batch of nothing has no shape.
    #[must_use]
    pub fn of(samples: &[Sample], device: &B::Device) -> Self {
        assert!(!samples.is_empty(), "a batch needs at least one sentence");
        let width = samples[0].tokens.len();

        let rows: Vec<i32> = samples
            .iter()
            .flat_map(|sample| {
                sample
                    .tokens
                    .iter()
                    .map(|row| i32::try_from(*row).unwrap_or(0))
            })
            .collect();
        let tags: Vec<i32> = samples
            .iter()
            .flat_map(|sample| {
                sample
                    .tags
                    .iter()
                    .map(|row| i32::try_from(*row).unwrap_or(0))
            })
            .collect();
        let verbs: Vec<i32> = samples
            .iter()
            .map(|sample| i32::try_from(sample.verb).unwrap_or(0))
            .collect();
        let real: Vec<bool> = samples
            .iter()
            .flat_map(|sample| (0..width).map(move |at| at < sample.length))
            .collect();

        let shape = [samples.len(), width];
        Self {
            tokens: Tensor::<B, 1, Int>::from_ints(rows.as_slice(), device).reshape(shape),
            tags: Tensor::<B, 1, Int>::from_ints(tags.as_slice(), device).reshape(shape),
            verbs: Tensor::<B, 1, Int>::from_ints(verbs.as_slice(), device),
            commands: Tensor::<B, 1, Int>::from_ints(
                samples
                    .iter()
                    .map(|sample| i32::from(sample.is_command()))
                    .collect::<Vec<i32>>()
                    .as_slice(),
                device,
            ),
            // Attention masks *padding*, so this is the inverse of `real`.
            pad: Tensor::<B, 1, Bool>::from_bool(
                burn::tensor::TensorData::from(
                    real.iter().map(|is| !is).collect::<Vec<bool>>().as_slice(),
                ),
                device,
            )
            .reshape(shape),
            real: Tensor::<B, 1, Bool>::from_bool(
                burn::tensor::TensorData::from(real.as_slice()),
                device,
            )
            .reshape(shape),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MAX_LEN, Vocabulary};
    use burn::backend::NdArray;

    type Cpu = NdArray<f32>;

    #[test]
    fn a_batch_has_a_row_per_sentence_and_a_column_per_word() {
        let vocabulary = Vocabulary::builtin();
        let samples: Vec<Sample> = ["what should i do next", "and now for something else"]
            .into_iter()
            .filter_map(|line| Sample::reject(line, &vocabulary))
            .collect();

        let device = burn::backend::ndarray::NdArrayDevice::default();
        let batch = Batch::<Cpu>::of(&samples, &device);

        assert_eq!(batch.tokens.dims(), [2, MAX_LEN]);
        assert_eq!(batch.tags.dims(), [2, MAX_LEN]);
        assert_eq!(batch.verbs.dims(), [2]);
    }

    #[test]
    fn padding_is_marked_and_it_is_most_of_the_buffer() {
        // The reason `real` exists: a five-word sentence is 27 rows of padding,
        // and a tagging score counted over all 32 would be mostly about those.
        let vocabulary = Vocabulary::builtin();
        let sample = Sample::reject("what should i do next", &vocabulary).expect("encodes");
        let device = burn::backend::ndarray::NdArrayDevice::default();
        let batch = Batch::<Cpu>::of(std::slice::from_ref(&sample), &device);

        let real: Vec<bool> = batch
            .real
            .into_data()
            .into_vec::<bool>()
            .expect("bools out");
        assert_eq!(real.iter().filter(|is| **is).count(), sample.length);
        assert!(
            sample.length * 2 < MAX_LEN,
            "padding should dominate, or this test proves nothing"
        );
    }
}
