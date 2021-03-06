//! Recurrent Neural Networks
use crate::{Device, Kind, Tensor};

// TODO: add some config for the various LSTM/GRU options.

pub trait RNN {
    type State;

    fn zero_state(&self, batch_dim: i64) -> Self::State;

    /// Applies a single step of the recurrent network.
    ///
    /// The input should have dimensions [batch_size, features].
    fn step(&self, input: &Tensor, state: &Self::State) -> Self::State;

    /// Applies multiple steps of the recurrent network.
    ///
    /// The input should have dimensions [batch_size, seq_len, features].
    fn seq(&self, input: &Tensor) -> (Tensor, Self::State);
}

pub struct LSTMState((Tensor, Tensor));

impl LSTMState {
    pub fn h(&self) -> Tensor {
        (self.0).0.shallow_clone()
    }

    pub fn c(&self) -> Tensor {
        (self.0).1.shallow_clone()
    }
}

/// A Long Short-Term Memory (LSTM) layer.
///
/// https://en.wikipedia.org/wiki/Long_short-term_memory
pub struct LSTM {
    w_ih: Tensor,
    w_hh: Tensor,
    b_ih: Tensor,
    b_hh: Tensor,
    hidden_dim: i64,
    device: Device,
}

impl LSTM {
    pub fn new(vs: &super::var_store::Path, in_dim: i64, hidden_dim: i64) -> LSTM {
        let gate_dim = 4 * hidden_dim;
        LSTM {
            w_ih: vs.kaiming_uniform("w_ih", &[gate_dim, in_dim]),
            w_hh: vs.kaiming_uniform("w_hh", &[gate_dim, hidden_dim]),
            b_ih: vs.zeros("b_ih", &[gate_dim]),
            b_hh: vs.zeros("b_hh", &[gate_dim]),
            hidden_dim,
            device: vs.device(),
        }
    }
}

impl RNN for LSTM {
    type State = LSTMState;

    fn zero_state(&self, batch_dim: i64) -> LSTMState {
        let shape = [batch_dim, self.hidden_dim];
        let zeros = Tensor::zeros(&shape, (Kind::Float, self.device));
        LSTMState((zeros.shallow_clone(), zeros.shallow_clone()))
    }

    fn step(&self, input: &Tensor, in_state: &LSTMState) -> LSTMState {
        let LSTMState((h, c)) = in_state;
        let (h, c) = input.lstm_cell(
            &[h, c],
            &self.w_ih,
            &self.w_hh,
            Some(&self.b_ih),
            Some(&self.b_hh),
        );
        LSTMState((h, c))
    }

    fn seq(&self, input: &Tensor) -> (Tensor, LSTMState) {
        let batch_dim = input.size()[0];
        let shape = [1, batch_dim, self.hidden_dim];
        let zeros = Tensor::zeros(&shape, (Kind::Float, self.device));
        let (output, h, c) = input.lstm(
            &[&zeros, &zeros],
            &[&self.w_ih, &self.w_hh, &self.b_ih, &self.b_hh],
            /*has_biases=*/ true,
            /*num_layers=*/ 1,
            /*dropout=*/ 0.,
            /*train=*/ false,
            /*bidirectional=*/ false,
            /*batch_first=*/ true,
        );
        (output, LSTMState((h, c)))
    }
}

pub struct GRUState(Tensor);

/// A Gated Recurrent Unit (GRU) layer.
///
/// https://en.wikipedia.org/wiki/Gated_recurrent_unit
pub struct GRU {
    w_ih: Tensor,
    w_hh: Tensor,
    b_ih: Tensor,
    b_hh: Tensor,
    hidden_dim: i64,
    device: Device,
}

impl GRU {
    pub fn new(vs: &super::var_store::Path, in_dim: i64, hidden_dim: i64) -> GRU {
        let gate_dim = 3 * hidden_dim;
        GRU {
            w_ih: vs.kaiming_uniform("w_ih", &[gate_dim, in_dim]),
            w_hh: vs.kaiming_uniform("w_hh", &[gate_dim, hidden_dim]),
            b_ih: vs.zeros("b_ih", &[gate_dim]),
            b_hh: vs.zeros("b_hh", &[gate_dim]),
            hidden_dim,
            device: vs.device(),
        }
    }
}

impl RNN for GRU {
    type State = GRUState;

    fn zero_state(&self, batch_dim: i64) -> GRUState {
        let shape = [batch_dim, self.hidden_dim];
        GRUState(Tensor::zeros(&shape, (Kind::Float, self.device)))
    }

    fn step(&self, input: &Tensor, in_state: &GRUState) -> GRUState {
        let GRUState(h) = in_state;
        let h = input.gru_cell(
            &h,
            &self.w_ih,
            &self.w_hh,
            Some(&self.b_ih),
            Some(&self.b_hh),
        );
        GRUState(h)
    }

    fn seq(&self, input: &Tensor) -> (Tensor, GRUState) {
        let batch_dim = input.size()[0];
        let shape = [1, batch_dim, self.hidden_dim];
        let zeros = Tensor::zeros(&shape, (Kind::Float, self.device));
        let (output, h) = input.gru(
            &zeros,
            &[&self.w_ih, &self.w_hh, &self.b_ih, &self.b_hh],
            /*has_biases=*/ true,
            /*num_layers=*/ 1,
            /*dropout=*/ 0.,
            /*train=*/ false,
            /*bidirectional=*/ false,
            /*batch_first=*/ true,
        );
        (output, GRUState(h))
    }
}
