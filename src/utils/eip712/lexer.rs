//!
//!  Lookup table layout
//!  ===================
//!
//!  ```text
//!  EOF    ;      :      ,      .      (      )      {      }      [      ]      =>
//!  IDENT  BLTIN  CONTR  LIB    IFACE  ENUM   STRUCT MODIF  EVENT  FUNCT  VAR    ANON
//!  AS     ASM    BREAK  CONST  CONTIN DO     DELETE ELSE   EXTERN FOR    HEX    IF
//!  INDEX  INTERN IMPORT IS     MAP    MEM    NEW    PAY    PULIC  PRAGMA PRIV   PURE
//!  RET    RETNS  STORAG SUPER  THIS   THROW  USING  VIEW   WHILE  RESERV T_BOOL T_ADDR
//!  T_STR  T_BYT  T_BYTS T_INT  T_UINT T_FIX  T_UFIX L_TRUE L_FALS L_HEX  L_INT  L_RAT
//!  L_STR  E_ETH  E_FINN E_SZAB E_WEI  T_YEAR T_WEEK T_DAYS T_HOUR T_MIN  T_SEC  :=
//!  =:     ++     --     !      ~      *      /      %      **     +      -      <<
//!  >>     <      <=     >      >=     ==     !=     &      ^      |      &&     ||
//!  ?      =      +=     -=     *=     /=     %=     <<=    >>=    &=     ^=     |=
//!  ERRTOK ERREOF
//!  ```
//!

use logos::{Lexer, Logos};
#[derive(Default, Clone, Copy)]
pub struct TypeSize(pub u8, pub u8);

#[derive(Debug, PartialEq, Clone, Copy, Logos)]
#[logos(extras = TypeSize)]
pub enum Token {
    #[regex("[a-zA-Z_$][a-zA-Z0-9_$]*")]
    Identifier,

    #[regex("bytes1|bytes[1-2][0-9]?|bytes3[0-2]?|bytes[4-9]", validate_bytes)]
    TypeByte,

    #[token("bytes")]
    TypeBytes,

    #[token("bool")]
    TypeBool,

    #[regex("uint(8|16|24|32|40|48|56|64|72|80|88|96|104|112|120|128|136|144|152|160|168|176|184|192|200|208|216|224|232|240|248|256)", default_size)]
    TypeUint,

    #[regex("int(8|16|24|32|40|48|56|64|72|80|88|96|104|112|120|128|136|144|152|160|168|176|184|192|200|208|216|224|232|240|248|256)", default_size)]
    TypeInt,

    #[token("string")]
    TypeString,

    //
    #[token("address")]
    TypeAddress,

    //
    #[regex("[0-9]+")]
    LiteralInteger,

    #[token("[")]
    BracketOpen,

    //
    #[token("]")]
    BracketClose,
}

fn validate_bytes(lex: &mut Lexer<Token>) {
    let slice = lex.slice().as_bytes();

    if slice.len() > 5 {
        lex.extras.0 = slice[5] - b'0';

        if let Some(byte) = slice.get(6) {
            lex.extras.0 = lex.extras.0 * 10 + (byte - b'0');
        }
    } else {
        lex.extras.0 = 1;
    }
}

fn default_size(lex: &mut Lexer<Token>) {
    lex.extras.0 = 32;
}
