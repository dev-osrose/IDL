#pragma once

/* Generated with IDL v0.1.4 */


#include "packetfactory.h"
#include <bitset>

namespace RoseCommon {
namespace Packet {

class SrvTest : public CRosePacket {
    public:
        static constexpr ePacketType PACKET_ID = ePacketType::PAKWC_TEST;
        SrvTest();
        SrvTest(CRoseReader reader);
        SrvTest(SrvTest&&) = default;
        SrvTest& operator=(SrvTest&&) = default;
        ~SrvTest() = default;
        
        static constexpr size_t size();
        
        
        struct Pote : public ISerialize {
            virtual bool read(CRoseReader&) override;
            virtual bool write(CRoseBasePolicy&) const override;
            
            static constexpr size_t size();
            
            Pote& set_a(const uint8_t);
            uint8_t get_a() const;
            Pote& set_b(const uint8_t);
            uint8_t get_b() const;
            Pote& set_c(const uint8_t);
            uint8_t get_c() const;
            
            private:
                union {
                    PACK(struct {
                        uint8_t a : 1;
                        uint8_t b : 7;
                    });
                    uint8_t c;
                } data;
        };
        
        struct Pote2 : public ISerialize {
            virtual bool read(CRoseReader&) override;
            virtual bool write(CRoseBasePolicy&) const override;
            
            static constexpr size_t size();
            
            Pote2& set_a(const uint8_t);
            uint8_t get_a() const;
            Pote2& set_b(const uint8_t);
            uint8_t get_b() const;
            
            private:
                std::bitset<8> bitset3;
        };
        
        
        SrvTest& set_a(const uint32_t);
        uint32_t get_a() const;
        SrvTest& set_b(const uint32_t);
        uint32_t get_b() const;
        SrvTest& set_c(const uint32_t);
        uint32_t get_c() const;
        SrvTest& set_d(const uint32_t);
        uint32_t get_d() const;
        SrvTest& set_e(const uint32_t);
        uint32_t get_e() const;
        SrvTest& set_f(const uint32_t);
        uint32_t get_f() const;
        SrvTest& set_g(const uint32_t);
        uint32_t get_g() const;
        SrvTest& set_h(const uint32_t);
        uint32_t get_h() const;
        SrvTest& set_pote(const Pote);
        Pote get_pote() const;
        SrvTest& set_pote2(const Pote2);
        Pote2 get_pote2() const;
        
        
        static SrvTest create(const uint32_t& a, const uint32_t& b, const uint32_t& c, const uint32_t& d, const uint32_t& e, const uint32_t& f, const uint32_t& g, const uint32_t& h, const Pote& pote, const Pote2& pote2);
        static SrvTest create(const uint8_t* buffer);
        static std::unique_ptr<SrvTest> allocate(const uint8_t* buffer);
    
    protected:
        virtual bool pack(CRoseBasePolicy&) const override;
    
    private:
        std::bitset<8> bitset1;
        uint32_t c;
        std::bitset<48> bitset2;
        Pote pote;
        Pote2 pote2;
};

}
}
