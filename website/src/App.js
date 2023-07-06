import logo from './logo.svg';
import './App.css';
import React from 'react';
import Histogram from './Histogram';
import data from './data.json';


function App() {
  return (
    <div>
      <h1>Histogram</h1>
      <Histogram data={data} />
    </div>
  );
}

export default App;
