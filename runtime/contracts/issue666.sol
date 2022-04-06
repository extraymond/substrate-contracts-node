contract Flip {
    function flip () pure public {
    }
}

contract Inc {
    Flip _flipper;

    event Howl();

    constructor (Flip _flipperContract) {
    	_flipper = _flipperContract;
    }

    function superFlip () public {

	    _flipper.flip();
        emit Howl();
    }
}
