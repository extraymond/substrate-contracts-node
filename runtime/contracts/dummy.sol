contract dummy {

    event SpitAnswer(uint256 output, uint256 secondOut);
    event SpitAnswerAgain(uint256 output);


    function plusOne(uint256 input) public returns(uint256) {
        emit SpitAnswer(input +1, input+2);
        emit SpitAnswerAgain(input +2);

        return input +1;
    }
}